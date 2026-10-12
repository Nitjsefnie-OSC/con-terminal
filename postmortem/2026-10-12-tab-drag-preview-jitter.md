# Horizontal Tab Drag Preview and Reorder Jitter

## What Happened

After the compact tab redesign, dragged tabs still used a fixed 180-by-28-point
rectangular preview. The user also reported a neighbouring tab oscillating
when dragging near the right end of the strip.

## Root Cause

The floating preview was a separate render path with old dimensions, padding,
font sizing, and a four-point radius. Updating resting tabs did not update it.

Reordering had multiple writers: individual tab hover, the tray, its trailing
drop target, and workspace mouse movement. Some used the pointer; others used
the floating preview's centre. Their answers could conflict within one gesture.
The bounds-based solver also treated visual insertion indices as original-order
drop slots and included the moved source placeholder in neighbour comparisons.
A preview repaint could therefore move the preceding tab back across the probe
and reverse the just-requested ordering without further pointer movement.

## Fix

Horizontal drag state carries the measured painted source size and its actual
visual index. Prepaint updates that index together with the rendered bounds;
it is not inferred from a newer, unpainted drop request. The floating capsule
uses the compact surface geometry and UI typography.

Only the workspace-root drag-move handler updates horizontal reorder targets.
The solver excludes the source placeholder, compares the remaining neighbours,
and maps the insertion back to original-order drop-slot coordinates. Missing
or partial layout snapshots retain the last valid target. Existing pane-to-tab
promotion and normal pane-resize mouse handling remain separate.

## Verification

Regression tests cover a probe inside the source's final gap, repeated repaints
with stationary probes and unequal tab widths, and invalid layout snapshots.
GPUI tests cover measured preview widths, centred identity glyphs at larger UI
font sizes, hover-close alignment, and unchanged label/click behavior.
Local runtime inspection is on macOS; Windows/Linux need their native CI and
hardware verification. No cross-compilation or terminal backend changes are
part of this fix.

The local workspace test set passed (159 tests). The final typography edit was
covered by a further seven titlebar tests, and the app plus companion CLI built
successfully. Native checks exercised reorder to the right end, return to the
original order, repeated end-tab drags, and light/dark appearance. These checks
do not replace Windows/Linux hardware validation or a frame-by-frame recording.

An additional strict Clippy run encountered existing lints in untouched files;
the debug-selector lint in this PR was corrected. A clean strict Clippy run is
not claimed.

## Lessons

Drag previews must share the resting component's geometry. One interaction
needs one authoritative target calculation, and layout feedback must be tested
over successive frames, not only with isolated cursor positions.
