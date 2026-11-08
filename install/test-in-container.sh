#!/bin/sh
# Installs Cuvée from a directory of release files in a fresh distribution
# container, the way a user's `curl | sh` would, and checks the result. CI runs
# it in Ubuntu, Debian, Fedora, Arch and openSUSE containers (as root, without
# KVM or a GPU, so the host steps are skipped).
#
#   docker run --rm -v "$PWD:/src:ro" -w /src fedora:latest sh install/test-in-container.sh /src/dist
#
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu
DIST="${1:?usage: test-in-container.sh DIST_DIR}"
DIR="$(cd "$(dirname "$0")" && pwd)"

# shellcheck disable=SC1091
echo "== $(. /etc/os-release && echo "$PRETTY_NAME")"
CUVEE_DIST_DIR="$DIST" sh "$DIR/install.sh" --yes --skip-host

BIN="$HOME/.local/bin"
RT="$HOME/.local/share/cuvee/runtime/current"
export PATH="$BIN:$PATH"

cuvee --version
"$RT/qemu/bin/qemu-system-x86_64" --version
"$RT/qemu/bin/qemu-img" --version
"$RT/qemu/bin/qemu-system-x86_64" -display help | grep -qx gtk
"$RT/bin/reims-util" --version
cuvee help-reims >/dev/null
cuvee list >/dev/null

# Every shared library each binary loads must be on this system.
fail=0
for b in "$RT/qemu/bin/qemu-system-x86_64" "$RT/qemu/bin/qemu-img" "$RT/bin/reims-ctl" \
  "$RT/bin/reims-util" "$BIN/cuvee-gui"; do
  if ldd "$b" | grep 'not found'; then
    echo "missing libraries for $b" >&2
    fail=1
  fi
done
[ "$fail" = 0 ]

# A second run changes nothing, and update sees the same release.
cuvee install --from "$DIST" --yes --skip-host | grep 'Runtime local' >/dev/null
cuvee update --from "$DIST" >/dev/null

# doctor runs (it reports problems here: a container has no KVM).
cuvee doctor --no-reims || true

cuvee uninstall
[ ! -e "$BIN/cuvee" ] && [ ! -e "$HOME/.local/share/cuvee/runtime" ]
echo "== ok"
