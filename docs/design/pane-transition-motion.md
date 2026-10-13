# Pane Transition Motion

Status: horizontal tab slot motion is implemented. Pane body motion is being
validated on the `feat/pane-presentation-motion` branch, not yet shipped.

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

Keep the terminal body flush to its pane. Do not introduce a card, rounded
terminal viewport, shadow, or additional inset: these would reduce usable space
and change selection, image and native-layer clipping. The native-app quality
comes from coherent motion and carefully aligned chrome, not a new frame around
terminal text.

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

### Live Presentation Contract

The implementation uses live content rather than immutable snapshots. One
workspace owns a numeric presentation tree keyed by session and pane identity.
It holds no terminal entity, process, history or image. Closing a pane closes
its terminal immediately; the departing region becomes a short-lived empty
terminal-colored region, not a screenshot of a dead terminal.

Commit the final logical tree and terminal size once. Its presentation tree
animates split ratios and divider gaps using one timestamp. Derive both child
rectangles from the same parent and snap their edges to display pixels.
Interpolating leaf rectangles independently is incorrect: consecutive closes
of nested panes can leave gaps when their parent is expanding too.

Live bodies lay out at their final pixel size throughout the transition and
move/crop, never stretch. Headers follow the current presentation width. A
shorter viewport crops from the top to keep the prompt visible. Growth beyond
the final body paints only the uncovered margins using the configured terminal
background and its original opacity. Never paint this fill behind live pixels:
doing so changes the apparent transparency by compositing it twice.

Windows and Linux use GPUI content masks. macOS also updates its native
NSScrollView viewport, document/surface origins and frame-scoped presentation.
Moving or cropping an unchanged native surface must not force a synchronous
Ghostty draw on every animation frame. Initialization, terminal size/scale
changes and explicit visibility handoffs retain their existing draw behavior.
macOS 12 and earlier settle immediately and retain the legacy native path.

Reduce Motion, divider dragging, window/display resizing, workspace or surface
switching and unrelated topology imports settle immediately. Split, close and
zoom reversals retarget from the displayed ratios. Settled dead branches are
pruned independently, including during another active transition. Idle renders
reuse the descriptor and presentation rectangles and request no animation frame.

### Validation and Follow-up

- Geometry tests cover nested and interrupted split/close/zoom, scale changes,
  fixed body sizes, topology changes, reduced motion and idle frame demand.
  A GPUI element fixture checks real layout at intermediate timestamps rather
  than testing only the numeric coordinator.
- Run Windows/Linux native CI and real-machine checks when available. Shared
  layout tests are not proof of native PTY, GPU, IME or selection behavior.
- Immutable snapshots remain an optional future refinement for departing
  content, not a prerequisite or an undocumented lease on a live GPU target.
- Inspect recorded intermediate frames with glass enabled and a bright desktop
  behind the app. Settled screenshots alone are insufficient acceptance.
