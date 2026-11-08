#!/usr/bin/env bash
# Runs build.sh inside ubuntu:22.04 (docker or podman), with nfpm and apt-file.
# Same arguments as build.sh; paths must be inside the repository.
#
#   packaging/native/container.sh --version 0.3.0 --cli dist/cuvee ... --out dist/packages
#
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NFPM_VERSION="${NFPM_VERSION:-2.48.0}"
ENGINE="${ENGINE:-$(command -v podman || command -v docker || true)}"
[[ -n $ENGINE ]] || { echo "container.sh: needs docker or podman" >&2; exit 1; }

# Run as the calling user so dist/ stays theirs (podman maps root to the user already).
user=()
[[ $(basename "$ENGINE") == podman ]] || user=(--user 0:0)

exec "$ENGINE" run --rm "${user[@]}" -v "$REPO:/src" -w /src \
  -e NFPM_VERSION="$NFPM_VERSION" -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" \
  ubuntu:22.04 bash -euc '
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq --no-install-recommends apt-file binutils ca-certificates curl gettext-base >/dev/null
    apt-file update >/dev/null
    curl -fsSL "https://github.com/goreleaser/nfpm/releases/download/v${NFPM_VERSION}/nfpm_${NFPM_VERSION}_Linux_x86_64.tar.gz" |
      tar -xz -C /usr/local/bin nfpm
    packaging/native/build.sh "$@"
    chown -R "$HOST_UID:$HOST_GID" dist 2>/dev/null || true
  ' build "$@"
