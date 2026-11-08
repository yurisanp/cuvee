#!/usr/bin/env bash
# Builds Cuvée's .deb and .rpm packages (cuvee and cuvee-runtime) with nfpm.
#
# SPDX-License-Identifier: GPL-3.0-or-later
#
#   packaging/native/build.sh --version 0.3.0 \
#     --cli dist/cuvee-x86_64-linux-musl --gui dist/cuvee-gui-x86_64-linux \
#     --runtime dist/cuvee-runtime-x86_64-linux.tar.gz --out dist/packages
#
# The dependencies are not written by hand: they come from the shared libraries
# each binary needs (its ELF NEEDED entries, plus the ones the app opens with
# dlopen). The .rpm asks for them by soname, which works on Fedora and
# openSUSE alike. The .deb needs package names, so this runs on Ubuntu 22.04,
# the oldest supported base, and asks apt-file which package ships each
# library. Libraries renamed by Debian's 64-bit time_t transition get the new
# name as an alternative ("libglib2.0-0 | libglib2.0-0t64").
#
# Needs: nfpm, readelf (binutils), apt-file with its index (apt-file update).
# packaging/native/container.sh runs this inside ubuntu:22.04 with all of it.

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$REPO/packaging/native"
VERSION='' RELEASE=1 CLI='' GUI='' RUNTIME='' OUT="$REPO/dist/packages"
FORMATS=(deb rpm)

die() { printf 'build.sh: %s\n' "$*" >&2; exit 1; }
usage() { sed -n '4,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

while (($#)); do
  case "$1" in
    --version) VERSION="${2#v}"; shift 2 ;;
    --release) RELEASE="$2"; shift 2 ;;
    --cli) CLI="$2"; shift 2 ;;
    --gui) GUI="$2"; shift 2 ;;
    --runtime) RUNTIME="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --format) IFS=, read -ra FORMATS <<<"$2"; shift 2 ;;
    -h | --help) usage ;;
    *) die "unknown option $1" ;;
  esac
done
[[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+([~+.][0-9A-Za-z.]+)?$ ]] || die "--version X.Y.Z is required"
for f in "$CLI" "$GUI" "$RUNTIME"; do
  [[ -n $f && -f $f ]] || die "--cli, --gui and --runtime must name existing files (got '$f')"
done
command -v nfpm >/dev/null || die "nfpm not found"
command -v readelf >/dev/null || die "readelf not found (binutils)"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
STAGE="$WORK/stage"
mkdir -p "$STAGE/runtime" "$OUT"
install -m755 "$CLI" "$STAGE/cuvee"
install -m755 "$GUI" "$STAGE/cuvee-gui"
tar -xf "$RUNTIME" -C "$STAGE/runtime"
# Accept a tarball with one top-level directory as well as a flat one.
if [[ ! -d $STAGE/runtime/qemu ]]; then
  top=("$STAGE"/runtime/*)
  ((${#top[@]} == 1)) && [[ -d ${top[0]}/qemu ]] || die "runtime tarball has no qemu/ directory"
  mv "${top[0]}"/* "$STAGE/runtime/" && rmdir "${top[0]}"
fi
for f in qemu/bin/qemu-system-x86_64 bin/reims-ctl bin/reims-util; do
  [[ -x $STAGE/runtime/$f ]] || die "runtime tarball has no $f"
done
find "$STAGE/runtime/qemu" -name ReimsVgpu.rom -type f | grep -q . || die "runtime has no ReimsVgpu.rom"
[[ -f $STAGE/runtime/runtime.toml ]] || echo "# upstream commits not recorded" >"$STAGE/runtime/runtime.toml"
mkdir -p "$STAGE/runtime/licenses"

is_elf() { [[ $(head -c4 "$1" | od -An -c | tr -d ' ') == '177ELF' ]]; }

# The sonames the ELF files under $1 (a file or a directory) need.
needed() {
  local f
  find "$@" -type f -print0 | while IFS= read -r -d '' f; do
    is_elf "$f" || continue
    readelf -d "$f" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p'
  done | sort -u
}

# The newest glibc symbol version the ELF files under $1 use (e.g. 2.35).
glibc_min() {
  local f
  find "$@" -type f -print0 | while IFS= read -r -d '' f; do
    is_elf "$f" || continue
    readelf -V "$f" 2>/dev/null | grep -o 'GLIBC_[0-9][0-9.]*' || true
  done | sed 's/GLIBC_//' | sort -Vu | tail -1
}

# The Debian package that ships a soname, from apt-file's index.
deb_package() {
  local re pkgs
  re="/lib(64|/x86_64-linux-gnu)/${1//./\\.}\$"
  pkgs="$(apt-file search -x -l "$re" 2>/dev/null | grep -Ev -- '-(dbg|dev)$|^lib32|^libx32' || true)"
  [[ -n $pkgs ]] || return 1
  # Several packages can ship the same file (libc6 and libc6-amd64 on some bases); the shortest wins.
  awk '{ print length, $0 }' <<<"$pkgs" | sort -n | head -1 | cut -d' ' -f2-
}

# One "Depends" entry: renamed-by-t64 libraries accept both names.
deb_entry() {
  case "$1" in
    libc6 | libgcc-s1 | libstdc++6 | zlib1g | lib*t64) echo "$1" ;;
    lib*) echo "$1 | ${1}t64" ;;
    *) echo "$1" ;;
  esac
}

# Writes nfpm's overrides (depends and recommends for deb and rpm) from the
# sonames on stdin. Arguments: newest glibc, then extra deb depends, extra rpm
# depends, deb recommends, rpm recommends (each newline separated, may be "").
overrides() {
  local glibc="$1" soname pkg e missing=()
  local -a deb=() rpm=()
  while IFS= read -r soname; do
    [[ -n $soname ]] || continue
    rpm+=("$soname()(64bit)")
    if [[ " ${FORMATS[*]} " == *" deb "* ]]; then
      if pkg="$(deb_package "$soname")"; then
        deb+=("$(deb_entry "$pkg")")
      else
        missing+=("$soname")
      fi
    fi
  done
  ((${#missing[@]} == 0)) || die "no Debian package found for: ${missing[*]} (run apt-file update?)"
  if [[ -n $glibc ]]; then
    for e in "${!deb[@]}"; do [[ ${deb[e]} != libc6 ]] || unset 'deb[e]'; done
    deb+=("libc6 (>= $glibc)") rpm+=("libc.so.6(GLIBC_$glibc)(64bit)")
  fi
  while IFS= read -r e; do [[ -n $e ]] && deb+=("$e"); done <<<"$2"
  while IFS= read -r e; do [[ -n $e ]] && rpm+=("$e"); done <<<"$3"
  list() { printf '      - "%s"\n' "$@" | awk '!seen[$0]++'; }
  echo 'overrides:'
  echo '  deb:'
  echo '    depends:'
  ((${#deb[@]})) && list "${deb[@]}"
  if [[ -n $4 ]]; then echo '    recommends:'; mapfile -t e_ <<<"$4"; list "${e_[@]}"; fi
  echo '  rpm:'
  echo '    depends:'
  list "${rpm[@]}"
  if [[ -n $5 ]]; then echo '    recommends:'; mapfile -t e_ <<<"$5"; list "${e_[@]}"; fi
}

# Libraries the app opens with dlopen (winit, wgpu): they are not NEEDED entries.
GUI_DLOPEN=(
  libxkbcommon.so.0 libxkbcommon-x11.so.0 libwayland-client.so.0 libwayland-cursor.so.0
  libwayland-egl.so.1 libvulkan.so.1 libEGL.so.1 libX11.so.6 libX11-xcb.so.1
  libXcursor.so.1 libXi.so.6 libXrandr.so.2
)

export CUVEE_VERSION="$VERSION" CUVEE_RELEASE="$RELEASE" STAGE REPO

VULKAN_DEB=$'mesa-vulkan-drivers\nvulkan-tools'
VULKAN_RPM=$'(mesa-vulkan-drivers or libvulkan_radeon or libvulkan_intel)\nvulkan-tools'

{
  envsubst <"$HERE/cuvee-runtime.yaml"
  needed "$STAGE/runtime" |
    overrides "$(glibc_min "$STAGE/runtime")" "" "" "$VULKAN_DEB" "$VULKAN_RPM"
} >"$WORK/cuvee-runtime.yaml"

{
  envsubst <"$HERE/cuvee.yaml"
  { needed "$STAGE/cuvee" "$STAGE/cuvee-gui"; printf '%s\n' "${GUI_DLOPEN[@]}"; } | sort -u |
    overrides "$(glibc_min "$STAGE/cuvee" "$STAGE/cuvee-gui")" \
      "cuvee-runtime (= $VERSION-$RELEASE)" "cuvee-runtime = $VERSION-$RELEASE" \
      $'vulkan-tools\nxdg-user-dirs' $'vulkan-tools\nxdg-user-dirs'
} >"$WORK/cuvee.yaml"

for pkg in cuvee-runtime cuvee; do
  for fmt in "${FORMATS[@]}"; do
    nfpm package --config "$WORK/$pkg.yaml" --packager "$fmt" --target "$OUT/" >&2
  done
done
[[ -z ${KEEP_CONFIG:-} ]] || cp "$WORK"/cuvee.yaml "$WORK"/cuvee-runtime.yaml "$KEEP_CONFIG/"
ls -1 "$OUT"
