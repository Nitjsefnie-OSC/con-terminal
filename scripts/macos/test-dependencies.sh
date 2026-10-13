#!/usr/bin/env bash

set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/prepare-dependencies.sh"

run_case() (
  local name="$1" have_cmake="$2" brew_status="$3" cmake_status="$4" expected="$5"
  local installs=0 versions=0 status=0
  command() {
    [[ "$#" == "2" && "$1" == "-v" && "$2" == "cmake" ]] || exit 99
    [[ "$have_cmake" == "1" ]]
  }
  cmake() {
    [[ "$#" == "1" && "$1" == "--version" ]] || exit 99
    versions=$((versions + 1))
    return "$cmake_status"
  }
  brew() {
    [[ "$*" == 'install --force-bottle cmake' ]] || exit 99
    [[ "$HOMEBREW_NO_AUTO_UPDATE" == "1" && "$HOMEBREW_NO_INSTALL_UPGRADE" == "1" \
      && "$HOMEBREW_NO_INSTALL_CLEANUP" == "1" ]] || exit 99
    installs=$((installs + 1))
    return "$brew_status"
  }
  if ensure_cmake; then status=0; else status=$?; fi
  [[ "$status" == "$expected" ]] || { echo "$name: wrong status $status"; exit 1; }
  if [[ "$have_cmake" == "1" ]]; then
    [[ "$installs" == "0" && "$versions" == "1" ]] || exit 1
  elif [[ "$brew_status" == "0" ]]; then
    [[ "$installs" == "1" && "$versions" == "1" ]] || exit 1
  else
    [[ "$installs" == "1" && "$versions" == "0" ]] || exit 1
  fi
  printf 'PASS: %s\n' "$name"
)

run_case 'reuse installed CMake' 1 0 0 0
run_case 'missing CMake installs bottle without updates' 0 0 0 0
run_case 'unavailable bottle fails without source fallback' 0 19 0 19
run_case 'broken installed CMake fails instead of upgrading' 1 0 20 20
