#!/usr/bin/env bash

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"
setup_release_env

if [[ -z "${CON_PREBUILT_APP_ARCHIVE:-}" ]]; then
  exec "$SCRIPT_DIR/build-app.sh"
fi

require_cmd ditto
require_cmd lipo
[[ -f "$CON_PREBUILT_APP_ARCHIVE" ]] || fail "missing native app archive: $CON_PREBUILT_APP_ARCHIVE"
mkdir -p "$CON_DIST_ROOT"
staging="$(mktemp -d "$CON_DIST_ROOT/native-app.XXXXXX")"
trap 'rm -rf "$staging"' EXIT
ditto -x -k "$CON_PREBUILT_APP_ARCHIVE" "$staging"
app="$staging/$CON_APP_NAME.app"
plist="$app/Contents/Info.plist"
[[ -f "$plist" ]] || fail "native archive does not contain $CON_APP_NAME.app"

check_plist() {
  local actual
  actual="$(/usr/libexec/PlistBuddy -c "Print :$1" "$plist")"
  [[ "$actual" == "$2" ]] || fail "native bundle $1 mismatch: expected $2, got $actual"
}
check_plist CFBundleIdentifier "$CON_BUNDLE_ID"
check_plist ConReleaseChannel "$CON_CHANNEL"
check_plist CFBundleShortVersionString "$(derive_marketing_version "$CON_APP_VERSION")"
check_plist CFBundleVersion "$CON_BUILD_NUMBER"

for binary in con con-cli; do
  path="$app/Contents/MacOS/$binary"
  [[ -x "$path" ]] || fail "native bundle is missing executable $binary"
  [[ "$(lipo -archs "$path")" == "$CON_ARCH" ]] \
    || fail "native bundle $binary architecture does not match $CON_ARCH"
done
[[ -L "$app/Contents/MacOS/ghostty" && "$(readlink "$app/Contents/MacOS/ghostty")" == "con-cli" ]] \
  || fail "native bundle is missing the Ghostty CLI compatibility link"
[[ -d "$app/Contents/Frameworks/Sparkle.framework" ]] || fail "native bundle is missing Sparkle"

rm -rf "$CON_APP_BUNDLE_PATH"
mv "$app" "$CON_APP_BUNDLE_PATH"
log "Prepared native $CON_ARCH bundle without rebuilding: $CON_APP_BUNDLE_PATH"
