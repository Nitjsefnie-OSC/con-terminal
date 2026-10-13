# Pane Viewport Animation Failed Visual Acceptance

## What Happened

A local, unshipped pane animation prototype passed geometry and focused app
tests. The maintainer then reported a large white region during Cmd+D on macOS.
This violated the existing no-leak native presentation contract. The complete
animation integration was withdrawn; pane header polishing was retained.

## What We Established

The prototype resized live terminal content to the final grid immediately,
while animating a larger viewport toward that grid. Uncovered margins were
intentional, not retained old content. Closing and zooming also discarded some
visible content immediately instead of retaining a completed outgoing scene.
This is not the content continuity visible in the Rex reference recording.

The prototype changed native host/document/surface geometry and skipped the
synchronous Ghostty draw for some viewport-only changes. The existing GPUI
transaction API synchronizes presentation; it does not freeze live renderer
targets or provide an immutable completed terminal frame.

We did not capture the failing compositor frames, so the exact timing causing
the white region remains unproven. The user report establishes the regression,
not a new defect in the upstream fork.

## Resolution

Remove the viewport coordinator, native geometry overrides, draw-skipping path,
and portable integration. Restore the established geometry-then-draw ordering
and frame-scoped native presentation request. Do not hide the problem with an
opaque cover, enlarged divider, or fade that changes terminal transparency.

## Verification Scope

The original prototype's geometry tests, PTY resize-count observations and
settled screenshots were insufficient to validate temporal composition. They
must not be cited as proof of a leak-free animation or production readiness.
After withdrawal, 217 focused app tests passed: pane tree (19), workspace (177),
Ghostty view (17), and terminal geometry (4). Formatting and warning-denying
Clippy passed. The application source and portable CI configuration match the
pre-prototype commit exactly. These checks establish removal of the experiment,
not a guarantee against every compositor fault. Windows/Linux native machines
are unavailable, so no real-machine verification is claimed there.

## Follow-up

- Keep pane body motion disabled until a backend supplies retained completed
  content with correct alpha, ownership and frame-ready handoff.
- Validate rapid nested split/close/zoom and reversals in recorded intermediate
  frames with glass enabled and a bright background, not only at endpoints.
- Preserve truthful selection, IME, mouse reporting and agent geometry; bound
  resource retention and keep capture off ordinary output and idle paths.
- Verify each native backend before enabling animation on it. A shared layout
  unit test is not compositor or GPU validation.
