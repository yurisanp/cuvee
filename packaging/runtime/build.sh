#!/usr/bin/env bash
# Builds Cuvée's runtime: reims-vgpu's QEMU, reims-ctl and reims-util, as one
# relocatable tarball that runs on any distribution with glibc 2.35 or newer.
#
# SPDX-License-Identifier: GPL-3.0-or-later
#
#   packaging/runtime/build.sh OUT_DIR
#
# Meant for a clean ubuntu:22.04 container (as root), where CI runs it:
#
#   docker run --rm -v "$PWD:/src" -w /src ubuntu:22.04 packaging/runtime/build.sh /src/dist
#
# Writes OUT_DIR/cuvee-runtime-x86_64-linux.tar.gz holding
#
#   qemu/bin/{qemu-system-x86_64,qemu-img}   QEMU finds share/ relative to bin/
#   qemu/share/qemu/...                      firmware and option ROMs (ReimsVgpu.rom)
#   bin/reims-ctl  bin/reims-util
#   licenses/                                reims-vgpu, QEMU and reims-util licences
#   runtime.toml                             the upstream commits it was built from
#   NEEDED.txt                               the shared libraries each binary loads
#
# Environment: REIMS_VGPU_REV (default: the commit packaging/runtime/UPSTREAM
# pins), JOBS (default: all CPUs).
set -euo pipefail

OUT="${1:?usage: build.sh OUT_DIR}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=packaging/runtime/UPSTREAM
. "$HERE/UPSTREAM"
REV="${REIMS_VGPU_REV:-$REIMS_VGPU_COMMIT}"
WORK="${WORK:-/tmp/cuvee-runtime}"
JOBS="${JOBS:-$(nproc)}"
export DEBIAN_FRONTEND=noninteractive

say() { printf '\n==> %s\n' "$*"; }

# The build's packages. QEMU is configured with --without-default-features, so
# only what is enabled below reaches the binary, whatever else is installed.
# X11, Wayland, xkbcommon, EGL and DRM headers are for SDL3, which the library
# builds from source and which opens those libraries at run time (dlopen).
if command -v apt-get >/dev/null 2>&1; then
  say "Installing build dependencies"
  apt-get update -qq
  apt-get install -y -qq --no-install-recommends \
    build-essential git curl ca-certificates pkg-config python3 python3-venv ninja-build \
    flex bison bzip2 cmake clang lld binutils file \
    libglib2.0-dev libpixman-1-dev libslirp-dev zlib1g-dev libzstd-dev \
    libpulse-dev libasound2-dev libusb-1.0-0-dev libgtk-3-dev libvulkan-dev \
    libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxi-dev libxfixes-dev libxss-dev \
    libxtst-dev libxkbcommon-dev libwayland-dev libwayland-bin libdecor-0-dev \
    libegl-dev libgl-dev libdrm-dev libgbm-dev libdbus-1-dev libudev-dev >/dev/null
  # clang 14's LLVM plugin makes binutils' nm abort on the library's linked object
  # ("LLVM ERROR: Invalid encoding"); the build needs plain ELF tools.
  rm -f /usr/lib/bfd-plugins/LLVMgold*.so
fi

if ! command -v rustup >/dev/null 2>&1; then
  say "Installing rustup"
  curl --proto '=https' --tlsv1.2 -sSfo /tmp/rustup-init \
    https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init
  chmod +x /tmp/rustup-init
  /tmp/rustup-init -y --profile minimal --default-toolchain none --no-modify-path
fi
# shellcheck disable=SC1091
[[ -f $HOME/.cargo/env ]] && . "$HOME/.cargo/env"

say "Fetching reims-vgpu $REV"
mkdir -p "$WORK"
cd "$WORK"
if [[ ! -d reims-vgpu/.git ]]; then
  git init -q reims-vgpu
  git -C reims-vgpu remote add origin https://github.com/steelbrain/reims-vgpu.git
fi
git -C reims-vgpu fetch -q --depth 1 origin "$REV"
git -C reims-vgpu checkout -q --force FETCH_HEAD
git -C reims-vgpu submodule update --init --depth 1 vendor/qemu ||
  git -C reims-vgpu submodule update --init vendor/qemu
VGPU_COMMIT="$(git -C reims-vgpu rev-parse HEAD)"
QEMU_COMMIT="$(git -C reims-vgpu/vendor/qemu rev-parse HEAD)"

# reims-ctl runs reims-util at the commit it pins.
UTIL_COMMIT="$(sed -n 's/^pub const REV: &str = "\([0-9a-f]*\)";/\1/p' reims-vgpu/crates/reims-ctl/src/reims_util.rs)"
[[ -n $UTIL_COMMIT ]] || { echo "could not read reims-util's pinned commit" >&2; exit 1; }
if [[ ! -d reims-util/.git ]]; then
  git init -q reims-util
  git -C reims-util remote add origin https://github.com/steelbrain/reims-util.git
fi
git -C reims-util fetch -q --depth 1 origin "$UTIL_COMMIT"
git -C reims-util checkout -q --force FETCH_HEAD

cd "$WORK/reims-vgpu"
rustup toolchain install --profile minimal --no-self-update
TOOLCHAIN="$(rustup show active-toolchain | cut -d' ' -f1)"
export CARGO_TARGET_DIR="$WORK/target"
# Keep build paths out of the binaries (panic messages, debug info).
export RUSTFLAGS="--remap-path-prefix=$WORK=/usr/src/cuvee-runtime --remap-path-prefix=$HOME/.cargo=/usr/src/cargo"
export CFLAGS="-O2 -ffile-prefix-map=$WORK=/usr/src/cuvee-runtime"
export CXXFLAGS="$CFLAGS"

say "Building the reims-vgpu library (cargo xtask dist --platform vulkan)"
cargo xtask dist --platform vulkan
PC="$CARGO_TARGET_DIR/dist-vulkan/lib/pkgconfig"
[[ -f $PC/reims-vgpu.pc ]] || { echo "cargo xtask dist wrote no reims-vgpu.pc" >&2; exit 1; }

say "Configuring QEMU"
PREFIX=/cuvee-qemu
rm -rf "$WORK/qemu-build" "$WORK/stage"
mkdir -p "$WORK/qemu-build"
cd "$WORK/qemu-build"
# Only what reims-ctl uses, so the binary loads libraries every distribution
# ships under the same soname: KVM, the GTK window (VMware SVGA), user-mode
# networking (slirp), USB passthrough, PulseAudio and ALSA audio (PipeWire
# serves both), qcow2/zstd disks and qemu-img. Ubuntu 22.04's PipeWire is older
# than QEMU's pipewire backend wants; pipewire-pulse covers it.
PKG_CONFIG_PATH="$PC" "$WORK/reims-vgpu/vendor/qemu/configure" \
  --prefix="$PREFIX" \
  --target-list=x86_64-softmmu \
  --without-default-features \
  --enable-system --enable-tools --enable-kvm --enable-pvg \
  --enable-gtk --enable-pixman --enable-slirp --enable-libusb \
  --enable-pa --enable-alsa --audio-drv-list=pa,alsa \
  --enable-zstd \
  --disable-docs --disable-werror
ninja -j "$JOBS"
DESTDIR="$WORK/stage" ninja install >/dev/null
Q="$WORK/stage$PREFIX"
[[ -x $Q/bin/qemu-system-x86_64 ]] || { echo "QEMU's install has no qemu-system-x86_64" >&2; exit 1; }
find "$Q" -name ReimsVgpu.rom -type f | grep -q . || { echo "QEMU's install has no ReimsVgpu.rom" >&2; exit 1; }

say "Building reims-ctl"
cd "$WORK/reims-vgpu/crates/reims-ctl"
cargo build --release --locked

say "Building reims-util"
cd "$WORK/reims-util"
cargo +"$TOOLCHAIN" build --release --locked

say "Assembling the runtime"
R="$WORK/cuvee-runtime"
rm -rf "$R"
mkdir -p "$R/bin" "$R/licenses/qemu" "$R/licenses/reims-vgpu" "$R/licenses/reims-util"
cp -a "$Q" "$R/qemu"
# Only the emulator and qemu-img are run; the rest of QEMU's tools stay out.
find "$R/qemu/bin" -type f ! -name qemu-system-x86_64 ! -name qemu-img -delete
rm -rf "$R/qemu/include" "$R/qemu/libexec" "$R/qemu/share/doc" "$R/qemu/share/applications" "$R/qemu/share/icons"
strip --strip-unneeded "$R/qemu/bin/"* || true
install -m755 "$CARGO_TARGET_DIR/release/reims-ctl" "$R/bin/reims-ctl"
install -m755 "$CARGO_TARGET_DIR/release/reims-util" "$R/bin/reims-util"
strip --strip-unneeded "$R/bin/"* || true
cp "$WORK/reims-vgpu/vendor/qemu/COPYING" "$WORK/reims-vgpu/vendor/qemu/COPYING.LIB" "$R/licenses/qemu/"
cp -r "$WORK/reims-vgpu/LICENSE" "$WORK/reims-vgpu/LICENSES" "$R/licenses/reims-vgpu/"
cp -r "$WORK/reims-util/LICENSE" "$WORK/reims-util/LICENSES" "$R/licenses/reims-util/"

glibc="$(ldd --version | sed -n '1s/.* //p')"
cat >"$R/runtime.toml" <<EOF
# Cuvée runtime: what this tarball was built from.
reims_vgpu = "$VGPU_COMMIT"
qemu = "$QEMU_COMMIT"
reims_util = "$UTIL_COMMIT"
glibc = "$glibc"
built = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EOF
for b in qemu/bin/qemu-system-x86_64 qemu/bin/qemu-img bin/reims-ctl bin/reims-util; do
  printf '%s:' "$b"
  readelf -d "$R/$b" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/ \1/p' | tr -d '\n'
  echo
done >"$R/NEEDED.txt"
cat "$R/NEEDED.txt"

"$R/qemu/bin/qemu-system-x86_64" --version
"$R/qemu/bin/qemu-system-x86_64" -display help | grep -qx gtk || { echo "QEMU has no gtk display" >&2; exit 1; }

mkdir -p "$OUT"
tar -C "$R" --owner=0 --group=0 --numeric-owner -czf "$OUT/cuvee-runtime-x86_64-linux.tar.gz" .
ls -l "$OUT/cuvee-runtime-x86_64-linux.tar.gz"
