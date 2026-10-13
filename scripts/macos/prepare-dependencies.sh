#!/usr/bin/env bash

set -euo pipefail

ensure_cmake() {
  if command -v cmake >/dev/null 2>&1; then
    cmake --version
    return
  fi

  # Hosted runners normally provide CMake. Never upgrade it incidentally or
  # fall back to a lengthy source build when an Intel bottle is unavailable.
  HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_INSTALL_UPGRADE=1 \
    HOMEBREW_NO_INSTALL_CLEANUP=1 brew install --force-bottle cmake || return
  cmake --version
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  ensure_cmake
fi
