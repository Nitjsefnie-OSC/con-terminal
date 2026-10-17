# Flatpak AppStream composition rejected the updated icon

## What Happened

The main-branch Flatpak run 39023230197 failed on both architectures with
`icon-not-found` and `filters-but-no-output`. The failure was found during
beta.140 preparation, before the release tag was pushed.

## Root Cause

The updated Tail A6 artwork is 2060px square. The package generator used its
dimensions to install it under `hicolor/2060x2060`, which AppStream composition
did not discover. Successful native application builds did not exercise this
packaging path.

## Fix

Give Flatpak a standard 256px derivative without changing the canonical artwork
or other platform icons. Validate its PNG dimensions and canonical source hash
in portable CI and Flatpak builds. Keep native AppStream and bundle smoke checks.

## Verification and Lessons

The local PNG size/provenance tests and manifest validation cover the packaging
input. Native Flatpak composition is not available on this macOS host and must
be verified by the release workflow. Changing shared artwork requires checking
each packaging consumer, not just the application's icon picker.
