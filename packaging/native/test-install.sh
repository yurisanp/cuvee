#!/usr/bin/env bash
# Installs the built packages in a clean container of each supported distro
# and checks that every binary finds its libraries and starts.
#
#   packaging/native/test-install.sh dist/packages [image...]
#
# Default images: Ubuntu 22.04 and 24.04, Debian 12 and 13, Fedora 42 and 43,
# openSUSE Tumbleweed. Inside CI, set ENGINE=none and pass one image's family
# as the second argument ("deb" or "rpm") to test the running system instead.
#
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail
PKGS="$(cd "${1:?usage: test-install.sh DIR [image...]}" && pwd)"
shift
IMAGES=("$@")
((${#IMAGES[@]})) || IMAGES=(ubuntu:22.04 ubuntu:24.04 debian:12 debian:13 fedora:42 fedora:43 opensuse/tumbleweed)
ENGINE="${ENGINE:-$(command -v podman || command -v docker || true)}"

# The checks, run as root inside the system under test.
read -r -d '' CHECK <<'SH' || true
set -euo pipefail
. /etc/os-release
echo "== $PRETTY_NAME"
if command -v apt-get >/dev/null; then
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq /pkgs/cuvee-runtime_*.deb /pkgs/cuvee_*.deb >/dev/null
elif command -v dnf >/dev/null; then
  dnf install -y -q /pkgs/cuvee-runtime-*.rpm /pkgs/cuvee[.-][0-9x]*.rpm >/dev/null
elif command -v zypper >/dev/null; then
  zypper --non-interactive -q install --allow-unsigned-rpm /pkgs/cuvee-runtime-*.rpm /pkgs/cuvee[.-][0-9x]*.rpm >/dev/null
else
  echo "no package manager I know" >&2; exit 1
fi
fail=0
for b in /usr/lib/cuvee/qemu/bin/* /usr/bin/reims-ctl /usr/bin/reims-util /usr/bin/cuvee /usr/bin/cuvee-gui; do
  [[ -x $b ]] || continue
  if ldd "$b" 2>&1 | grep -q 'not found'; then
    echo "missing libraries for $b:"; ldd "$b" | grep 'not found'; fail=1
  fi
done
/usr/lib/cuvee/qemu/bin/qemu-system-x86_64 --version | head -1
/usr/lib/cuvee/qemu/bin/qemu-system-x86_64 -device help 2>/dev/null | grep -qi reims ||
  { echo "QEMU has no Reims vGPU device"; fail=1; }
reims-ctl --version
reims-util --version
cuvee --version
test "$(cat /usr/lib/cuvee/packaged)" = "$(command -v apt-get >/dev/null && echo deb || echo rpm)"
test -f /usr/lib/udev/rules.d/70-cuvee-udmabuf.rules
test -f /usr/lib/modules-load.d/cuvee.conf
getent group kvm >/dev/null
# Removing must leave nothing of ours behind.
if command -v apt-get >/dev/null; then apt-get remove -y -qq cuvee cuvee-runtime >/dev/null
elif command -v dnf >/dev/null; then dnf remove -y -q cuvee cuvee-runtime >/dev/null
else zypper --non-interactive -q remove cuvee cuvee-runtime >/dev/null; fi
if [[ -e /usr/lib/cuvee || -e /usr/bin/cuvee ]]; then echo "files left after removal:"; find /usr/lib/cuvee /usr/bin/cuvee 2>/dev/null | head; fail=1; fi
((fail == 0)) && echo "ok: $PRETTY_NAME"
exit "$fail"
SH

if [[ $ENGINE == none ]]; then
  [[ -d /pkgs ]] || ln -s "$PKGS" /pkgs
  exec bash -c "$CHECK"
fi
[[ -n $ENGINE ]] || { echo "test-install.sh: needs docker or podman" >&2; exit 1; }

failed=()
for img in "${IMAGES[@]}"; do
  if ! "$ENGINE" run --rm -v "$PKGS:/pkgs:ro" "$img" bash -c "$CHECK"; then
    failed+=("$img")
  fi
done
if ((${#failed[@]})); then
  echo "FAILED: ${failed[*]}" >&2
  exit 1
fi
echo "all ${#IMAGES[@]} distros ok"
