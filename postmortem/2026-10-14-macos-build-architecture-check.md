# Architecture Check Blocked Completed Native Builds

## Incident

[beta.132 macOS release](https://github.com/nowledge-co/con-terminal/actions/runs/38488469671)
failed after native compilation and bundle creation on both architectures.
The build job used `lipo -verify_arch "$CON_ARCH" "$binary"`. Apple's tool
interpreted the binary path as another architecture argument and exited 1.
Neither native bundle reached upload, so ARM packaging and publication were
skipped. This release did not reach notarization or ticket download.

The Intel dependency setup optimization did work: its dependency step completed
within the same second, instead of the previous 31-minute CMake source upgrade.
That does not excuse the subsequent regression.

## Root Cause

The new build-side check had a different command from the tested prebuilt
bundle check. Tests exercised `lipo -archs` in preparation, but not the separate
`-verify_arch` invocation embedded in the workflow. Workflow syntax validation
cannot detect a valid shell command with incorrectly ordered tool arguments.

## Fix and Verification

Use one `verify_native_binary_architecture` helper from both native build and
prebuilt preparation. Read `lipo -archs` and require the exact expected thin
architecture for both app and CLI. Preserve executable checks and reject unreadable
or mismatched binaries; do not remove the architecture gate.

Real native-tool fixtures run the shared build check on an extracted existing
Intel executable slice, then exercise archive handoff. Add rejection cases for
a universal CLI and a malformed executable. These tests require no compiler,
cross-compilation or Apple credentials. Native release acceptance remains
required for the full build/sign/notarize chain. Windows/Linux runtime and their
release workflows do not change.

## Lessons

Share build and packaging checks instead of embedding an untested variant in
YAML. A successful compile is not a successful artifact handoff. Keep immutable
tags unchanged: rerunning beta.132 alone will repeat its old command, not apply
the fix from main.
