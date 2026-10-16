# Sidebar Motion

Status: local preview, pending reporter acceptance and native Windows/Linux
verification. No dependency changes.

## Contract

Left and right sidebars use the same critically damped GPUI spring. Position
and velocity survive a reversal. The analytic upstream solver samples elapsed
time rather than counting frames; no timer or independent animation thread is
introduced. Settled sidebars request no animation frames. Reduce Motion settles
immediately without requesting another frame.

The compact/expanded vertical-tab rail uses the same spring in both directions,
including macOS. Keep outgoing session rows mounted until collapse settles.
The expanded tab navigator uses the full sidebar width: its tools move between
the vertical compact rail and a horizontal toolbar. Sample these positions,
row stride and reserved width from the same spring; do not crossfade two lists
or create a timer for each icon. Keep tab titles single-line and clipped as
they reveal, not wrapped repeatedly during the transition. The compact endpoint
must match the settled rail's icon geometry. The workspace samples the rail
motion so terminal reservation and the sidebar edge move together.
The critically damped response uses an angular frequency of 25 radians/second:
a gentle acceleration and a short, non-bouncing deceleration, not a long fade.

File, search and agent panel content keeps its full width behind a clipped viewport. The left
sidebar moves toward the left window edge; the right sidebar moves toward the
right edge. Agent content remains behind its existing cached view boundary,
with stable width and height. Do not fade text or narrow its layout on every
frame. Real content, theme, and size changes still invalidate the view.

Terminal reservation and sidebar viewport derive from the same progress.
macOS native terminal geometry continues to request frame-scoped presentation
synchronization. No opaque full-terminal animation cover is introduced and
Terminal Glass remains independent of Window Chrome. The bottom input bar uses
the same spring for its visible height, with full-height cached content behind
the clip. Its text does not fade or reflow during reveal. Focus, pane-picker
reveal, and the visibility toggle resolve to the same authoritative intent.
Tab-strip motion keeps its existing policy; this is not a global easing replacement.

Panel booleans remain the authoritative visibility intent for focus and saved
state. Resolve spring targets from them so Files, Search, keyboard shortcuts,
and agent-request reveal paths cannot leave stale motion state. Keep outgoing
content present until its geometry reaches the hidden endpoint. Resize handles
must not begin a drag against a moving sidebar edge.

## Acceptance

- Reverse each sidebar repeatedly, including both together and nested panes.
- Reverse the bottom bar, including a multiline draft, all three modes, and
  revealing it through focus or the pane picker. Hidden inputs must lose focus;
  revealing through focus must not wait for the animation to finish.
- Verify typing, focus restoration, Files/Search, provider/model controls, and
  opening the agent panel through a request.
- Check glass over a bright backdrop for intermediate-frame leaks, not merely
  settled screenshots. Check reduced-motion behavior and enlarged UI fonts.
- Verify viewport clipping and stable content width at intermediate extents.
- Measure actual frame delivery on available hardware before claiming 60/120
  FPS. Correct spring math is not evidence that terminal resizing is cheap.
- Run native Windows/Linux tests and visual acceptance when hardware is
  available; macOS evidence does not establish those results.
- Include internal and external displays, with the same display tested on
  battery and AC. A low reproduction rate on an external 60 Hz display does
  not validate an internal adaptive-refresh display. Record the actual display
  and power configuration rather than inferring Low Power Mode from battery use.

## Adjacent Controls

All three input modes render the same Textarea structure and inherit its native
editor inset. Do not add mode-specific outer padding: the editor already owns
the text inset, and an extra Agent-mode inset shifts the cursor on mode changes.
Shell overlays continue to align with the editor's native inset. Prose and
command font choices remain unchanged.

Mode and submit controls share their dimensions and rounding. Empty submit is
disabled; ready input has an accent fill. Use the official flattened Phosphor
paths for small glyphs rather than depending on SVG stroke rasterization.
The compact tab rail separates tool controls from session navigation with
space, not extra borders. Its drag fallback origin must include that space;
the layout test checks the origin against the actual first session bounds.
