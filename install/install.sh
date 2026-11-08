#!/bin/sh
# Cuvée's installer: macOS VMs on Linux, in one command, on any distribution.
#
#   curl -fsSL https://yurisanp.github.io/cuvee/install.sh | sh
#   curl -fsSL https://yurisanp.github.io/cuvee/install.sh | sh -s -- --yes
#
# It downloads the cuvee command (a static binary) from the newest GitHub release,
# checks its SHA-256 against the release's SHA256SUMS and runs `cuvee install`,
# which installs your distribution's packages (apt, dnf, pacman or zypper, through
# sudo), the runtime (QEMU with reims-vgpu, reims-ctl, reims-util) and the app in
# your home directory, prepares the host and runs `cuvee doctor`. Arguments after
# `--` go to `cuvee install` (cuvee install --help lists them).
#
# Environment: CUVEE_VERSION=v0.2.0 installs that release instead of the newest;
# CUVEE_DIST_DIR=DIR installs from a directory holding a release's files.
#
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu

REPO=yurisanp/cuvee
ASSET=cuvee-x86_64-linux-musl

say() { printf '%s\n' "$*"; }
die() {
  printf 'cuvee: %s\n' "$*" >&2
  exit 1
}

case "$(uname -s)/$(uname -m)" in
  Linux/x86_64 | Linux/amd64) ;;
  *) die "Cuvée runs on Linux on x86-64; this is $(uname -s) on $(uname -m)." ;;
esac
if [ "$(id -u)" = 0 ]; then
  say "Aviso: rodando como root. O Cuvée se instala na pasta pessoal de quem roda;"
  say "rode como seu usuário (sem sudo) para instalar para você. Ele pede sudo quando precisa."
fi

fetch() { # URL FILE
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --proto '=https' --retry 3 -o "$2" "$1"
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$2" "$1"
  else
    die "instale o curl ou o wget e rode de novo."
  fi
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d' ' -f1
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 -r "$1" | cut -d' ' -f1
  else
    die "sem sha256sum, shasum ou openssl para conferir o download."
  fi
}

tmp="$(mktemp -d "${TMPDIR:-/tmp}/cuvee-install.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT INT TERM

version="${CUVEE_VERSION:-latest}"
if [ -n "${CUVEE_DIST_DIR:-}" ]; then
  say "Cuvée: instalando de $CUVEE_DIST_DIR"
  cp "$CUVEE_DIST_DIR/$ASSET" "$CUVEE_DIST_DIR/SHA256SUMS" "$tmp/"
else
  if [ "$version" = latest ]; then
    base="https://github.com/$REPO/releases/latest/download"
  else
    base="https://github.com/$REPO/releases/download/$version"
  fi
  say "Cuvée: baixando o comando cuvee ($version)"
  fetch "$base/$ASSET" "$tmp/$ASSET" || die "não deu para baixar $base/$ASSET"
  fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS" || die "não deu para baixar $base/SHA256SUMS"
fi

want="$(awk -v f="$ASSET" '$2 == f || $2 == "*" f { print $1 }' "$tmp/SHA256SUMS")"
[ -n "$want" ] || die "o SHA256SUMS da release não tem $ASSET"
got="$(sha256 "$tmp/$ASSET")"
[ "$got" = "$want" ] || die "SHA-256 de $ASSET não confere (veio $got, a release diz $want)"
chmod +x "$tmp/$ASSET"

# Under `curl | sh` the script is stdin, so the package manager's questions are
# asked on the terminal instead, when there is one.
status=0
if [ -t 0 ]; then
  "$tmp/$ASSET" install "$@" || status=$?
elif (: </dev/tty) 2>/dev/null; then
  "$tmp/$ASSET" install "$@" </dev/tty || status=$?
else
  "$tmp/$ASSET" install "$@" </dev/null || status=$?
fi
exit "$status"
