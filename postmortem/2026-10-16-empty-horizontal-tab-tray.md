# Empty horizontal tab tray with restored vertical tabs

## What happened

A window restored with multiple vertical tabs showed an empty capsule in the
title bar. The vertical list worked, but the horizontal tray remained visible
without any tab content.

## Root Cause

Workspace construction initialized horizontal-strip motion from tab count
alone. Rendering correctly suppressed horizontal tab contents in vertical mode,
but still drew the tray while its motion value was nonzero. Subsequent visibility
updates knew about orientation; initialization did not.

## Fix

Construction and later updates now share the orientation-and-count visibility
rule. Restored vertical windows start with the horizontal strip fully closed.
The fix does not suppress legitimate horizontal exit animations or pane-to-tab
drag previews at render time.

## Verification

Regression coverage checks restored vertical tab counts of zero, one, two and
twelve, including a settled motion state, and horizontal single/multiple tabs.
All 60 affected sidebar, horizontal-tab, activity, asset, restore and motion tests passed,
as did Clippy with warnings denied and formatting checks. A native macOS preview
restored three vertical tabs without an empty titlebar tray; both Flexoki themes
and compact/expanded navigation were inspected. Windows/Linux desktop behavior
was not tested locally. No terminal renderer or platform-specific code changed.

## Lesson

Initial animation state is part of restored layout state. Derive it from the
same visibility policy used for subsequent transitions, not a partial proxy
such as the number of tabs.
