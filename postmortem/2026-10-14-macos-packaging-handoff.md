# Intel Ticket Downloads Failed Despite Bounded Retries

## Incident

[beta.131 macOS release](https://github.com/nowledge-co/con-terminal/actions/runs/38425958695)
again failed on the Intel runner. Compilation, signing and notarization succeeded,
but all four ticket download attempts returned `NSURLErrorDomain -1005` and
stapler exit 68. ARM packaging in that same run succeeded. The draft correctly
remained unpublished. The preceding retry fix worked as implemented but did
not resolve this persistent connection failure.

## Evidence and Limits

Repeated failures establish that brief backoff is insufficient on this runner
path. They do not identify the responsible Apple endpoint, transport feature,
GitHub network or runner image. A successful ARM package does not guarantee
that an Intel ticket download on ARM will succeed. That is a release acceptance
check, not a fact inferred from unit tests.

## Change

Keep architecture-specific native compilation. Transfer the complete unsigned
app bundle through a ZIP archive, preserving symlinks and permissions. Sign,
notarize, staple, validate and package both architectures on ARM runners,
separately from compilation. Check bundle identity, channel, version/build,
binary architectures, CLI link and Sparkle before accepting the transferred
bundle. Smoke-test the CLI natively during build and under Rosetta when packaging
Intel on ARM. Do not alter signatures, Apple credentials, bundle IDs or gates.

Publication and updater jobs download only finalized `macos-*` artifacts, never
internal `native-app-*` transfer archives. Packaging retries can reuse successful
native builds while artifacts remain available. Local releases still build when
no explicit prebuilt archive is provided. Windows/Linux workflows do not change.

## Verification and Recovery

The same Intel run spent 31 minutes 47 seconds in dependency setup. It already
had CMake 4.4.2; `brew update` followed by `brew install cmake` upgraded to 4.4.4
and compiled it from source for 29 minutes because no Intel bottle was available.
Reuse the installed tool, log its version, and require a bottle only when it is
missing. Four shell fixtures cover reuse, missing-tool installation, unavailable
bottles and an installed tool that fails its version check. Do not attribute this
setup delay to Zig (18 seconds in that run) or Apple notarization.

Native macOS fixtures exercise archive extraction, symlink/executable-bit
preservation, metadata and architecture mismatch, and missing CLI/link/framework.
They use an existing OS executable slice; no cross-compilation occurs. Bounded
stapler retry fixtures remain required. A dedicated macOS PR job checks the
handoff without release secrets.

The Apple-signed end-to-end run is still required to verify Intel tickets on ARM.
No public success or Apple outage is claimed before that run. Existing immutable
tags retain their original workflow and cannot inherit this split automatically.
Never move a release tag or publish an incomplete draft to work around it.
