# Pane Transition Motion

Status: horizontal tab slot motion is implemented. The pane viewport animation
prototype was withdrawn after a macOS split exposed a large white region.
Pane body transitions remain immediate; the native presentation sync is retained.

## Reference Direction

The maintainer's Rex reference recording (2026-10-13) shows the important
relationship: a newly split region takes space from its neighbor, and closing
it returns that space. Existing content remains legible throughout. This is a
spatial transition, not an independent fade on each pane. Match that continuity
without copying its visual design or assuming its rendering implementation.

For tabs, animate slot redistribution together with arrival/departure, rather
than fading a label in a slot that already jumped to its final size. For panes,
resolve the presentation-versus-grid contract below before choosing an easing
curve. A pleasing endpoint does not validate intermediate terminal frames.

## Current Boundaries

`PaneTree::render_node` lays out splits with flex bases and a one-point divider.
Splitting adds a leaf immediately; closing removes it immediately. macOS embeds
native terminal layers under GPUI. `GhosttyView::update_frame` synchronizes
backing dimensions and requests frame-scoped native presentation for geometry
changes. Windows `RenderSession::resize` resizes the renderer, VT, mouse geometry
and ConPTY. Linux `resize` also updates the VT and PTY.

Consequently, interpolating the real split ratio is not a cheap decorative
animation: it can repeatedly change the terminal grid and trigger TUI redraw.
The existing native presentation synchronization fixes compositor consistency,
not that application-level resize cost.

## Recommended Sequence

1. Keep per-tab chrome motion separate. Logical closure stays immediate;
   decorative exit state must never own a terminal process.
2. Establish a pane transition presentation layer with final logical bounds
   committed once. Keep cell geometry, selection and mouse reporting truthful.
3. Animate only presentation if the backend can provide a retained, correctly
   clipped frame without stretching text, readback per frame, or changing PTY
   size per frame. Disable interaction with an outgoing decorative frame.
4. Validate each native backend before enabling terminal-body motion there.
   No capable backend means an immediate transition, not an opaque cover,
   live-terminal fade or stretched screenshot fallback.

macOS may need an upstream presentation-only layer primitive; GPUI transforms
do not automatically transform its embedded AppKit terminal views. Windows and
Linux participate in GPUI's scene, but their retained terminal frames, resize
readiness and mouse-coordinate contracts still need explicit validation. The
current code does not establish that this primitive exists across all ports.

## Acceptance

Count PTY/grid resizes during a transition, inspect intermediate frames rather
than just endpoints, and measure a content-heavy TUI. Include nested splits,
rapid open/close, zoom, restored panes, IME, selection, transparency, multiple
displays and Reduce Motion. Idle windows must not request animation frames.

See [native terminal presentation](native-terminal-presentation.md) for the
existing macOS synchronization contract and its platform scope.

## Implementation Study: 2026-10-13

### Visual Hierarchy

Keep the terminal body flush to its pane. The later macOS
[workspace canvas](workspace-depth.md) introduces one inset and rounded outer
boundary for the whole pane tree, not a separate card around each split.
Its native clipping must preserve selection and images. It does not authorize
terminal-body animation or stretched retained frames.

The split-pane header remains 24 points high. Its title is centered independently
of the controls, with equal reserved space on both sides. Solid system-font ink
distinguishes focused and inactive panes without alpha-fading glyphs. A small,
opaque neutral fill change identifies focus. Zoom and close use component
buttons in fixed 20-point squares, centered vertically and revealed on header
hover; hiding them must not shift the title. A single terminal remains free of
pane chrome. Surface rails and editor file tabs keep their existing semantics.

### Retained Frames Are Not Yet a Shared Primitive

Source inspection establishes three different ownership contracts:

The macOS renderer references below match the resolved Ghostty revision
`8b0ac6c4bc2b0977ef81f80d0c189b2e540f4077`, not an older reference checkout:
[IOSurfaceLayer](https://github.com/wey-gu/ghostty/blob/8b0ac6c4bc2b0977ef81f80d0c189b2e540f4077/src/renderer/metal/IOSurfaceLayer.zig)
and [frame target reuse](https://github.com/wey-gu/ghostty/blob/8b0ac6c4bc2b0977ef81f80d0c189b2e540f4077/src/renderer/generic.zig).

- macOS: `IOSurfaceLayer.setSurface` retains an IOSurface long enough to assign
  it to CALayer contents. `generic.zig` cycles through swap-chain frame targets
  and replaces a target only when its size/configuration changes. Retaining
  `layer.contents` therefore extends lifetime, but does **not** establish an
  immutable snapshot. A later frame may write to that same target. Relying on
  every resize replacing every target is also incorrect: replacement happens
  lazily per frame, and a rapid reversal can return to the previous size.
- Windows: `cached_image` is an `Arc<RenderImage>` produced from a completed
  BGRA frame. It can be retained without a terminal entity. Atlas eviction must
  remain tied to its last decorative paint, not the live view's image queue.
- Linux: the renderer paints cached shaped cell rows, backgrounds and Kitty
  image layers into GPUI. It has no equivalent completed image of the entire
  terminal. Retaining only text or a view cache would omit image/background
  layers or retain live terminal state.

No terminal-body animation is enabled by this study. In particular, a Core
Animation transition on the live Ghostty layer cannot replace the missing
ownership and clipping contract. GPUI's transaction-presentation request makes
geometry and presentation atomic; it does not freeze renderer targets.

### Transition Contract

One workspace transition owns presentation records keyed by stable session,
pane and surface identity. It never owns a terminal process. Each record needs
the old and final pane rectangles, an immutable completed frame, its pixel size,
display scale and color/alpha interpretation. Resource ownership must explicitly
prevent overwrite as well as deallocation.

Commit the new logical tree and terminal size once. Animate a tiled set of
presentation rectangles with one timestamp and one easing curve; shared split
edges must be calculated once and snapped to the display pixel grid. Frames
move and crop at their original scale, never stretch. Chrome follows those same
rectangles. Only final, correctly sized live frames are revealed on handoff.
Do not make an outgoing decorative frame clickable or route mouse reporting
through an interpolated grid.

Reduce Motion, divider dragging, window/display resizing, workspace or surface
switching, and unsupported capture settle immediately. A second topology change
must either retarget from the currently displayed scene with valid immutable
resources or settle the first transition; it must not accumulate nested covers.
No capture or resource allocation belongs on the idle or ordinary output path.

### Remaining Work

- Do not re-enable the final-grid/live-viewport prototype. Resizing the live
  terminal first and interpolating its viewport leaves uncovered margins; it
  does not retain the old scene or establish frame readiness. Easing and a
  background fill cannot supply those missing guarantees.
- Establish an explicit immutable-frame capability per backend. For macOS,
  investigate an upstream renderer-owned snapshot or GPU-copy operation with
  completion ordering and bounded retention; do not read private renderer
  structures or expand the existing fork exception.
- Validate a Windows retained-image lease and Linux full-scene capture before
  enabling either backend. A shared trait must not pretend these are equivalent
  merely because all three can draw a terminal.
- Implement the presentation coordinator only once a backend satisfies that
  contract. Measure capture latency/memory and count PTY resizes for nested
  split, close and zoom, including rapid reversals.
- Inspect recorded intermediate frames with glass enabled and a bright desktop
  behind the app. Settled screenshots alone are insufficient acceptance.

### Withdrawn Prototype

The 2026-10-13 prototype committed the final terminal grid once and interpolated
split viewports. Unit tests established tiled geometry and bounded resize counts,
but the maintainer's Cmd+D test exposed a large white region. That is a failed
visual acceptance test, even though settled screenshots and geometry tests passed.

The prototype also skipped the synchronous Ghostty draw for some viewport-only
updates, changing the established native geometry-to-draw ordering. The exact
compositor timing behind the reported white region was not captured. Neither
the GPUI fork nor its transaction API can be blamed on this evidence.

The complete prototype was removed, including its native geometry overrides
and Windows/Linux integration. Pane chrome polishing remains. Future motion
must demonstrate retained content, correct alpha and frame-ready handoff before
it replaces the immediate, synchronized path. See the
[postmortem](../../postmortem/2026-10-13-pane-motion-coverage.md).
