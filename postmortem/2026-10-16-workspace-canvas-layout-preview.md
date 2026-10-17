# Workspace canvas preview: layout ownership

## What happened

The local macOS canvas preview gave the terminal a rounded frame but left the
composer below it in a separate chrome strip. User testing exposed an incoherent
boundary, unnecessary sidebar rules, missing sidebar controls in the caption,
and a terminal visible behind the returning horizontal caption row. The light
theme made excessive gutters and material differences particularly visible.
This is an unreleased prototype, not a production incident.

## Confirmed causes

- The canvas wrapper owned only the pane tree, not the terminal and input region.
- The old sidebar divider rules remained alongside the new canvas gutters.
- Caption docking removed the left-sidebar visibility control.
- Caption docking also used the saved pinned preference while a new-tab reveal
  temporarily forced a 44pt rail. The toolbar was therefore assigned to a
  sidebar that could not contain it. Interpolating its width across the terminal
  did not provide a valid containing rectangle during the transition.
- Changing to horizontal tabs retained the outgoing caption spring, even though
  horizontal tabs immediately needed the complete global row.
- A GPUI-colored safe-area extension showed a material step against the native
  terminal despite equal RGB and opacity inputs. The underlying color/compositing
  mechanism has not been established; equal parameters are not visual proof.
- An undecorated single terminal leaf returned before the new pane-mask
  prepaint callback. After a split was reduced to one pane, that surviving
  surface could keep its old half-sized mask even with a full-sized native
  frame. This explains the reported half-window cutout, unlike earlier cache
  hypotheses alone.
- Drop-preview bounds divided the full container without reserving the 1pt
  dividers rendered by flex layout. The preview also kept old rectangular
  fills and resize feedback despite rounded pane cards.

## Changes

The canvas now owns the terminal and its bottom safe area. The composer is a
centered overlay within that safe area, capped at 800pt, with terminal rows
reserved so output cannot hide underneath. Its popovers share its horizontal
geometry. A cached passive native material view supplies only the safe-area
background; it neither receives input nor covers terminal output. Native
presentation synchronization also covers changes to this view.

The macOS canvas gutter is 2pt with 8pt corners. Sidebar rules are removed,
the visibility control remains available, and horizontal mode immediately
restores the caption row. Windows/Linux retain their previous layout paths.
Caption eligibility now follows the effective rail/panel state and actual
available width. An independent caption spring has been removed: caption
progress is derived from the sidebar's width and visibility, so it cannot
continue a second animation after navigation has settled. Compact controls share a tighter spacing budget. The composer
is fixed in place and revealed through a clip; output reservation changes once
at reveal/fully-covered boundaries instead of resizing the PTY on every frame.

That initial reveal contract still prepared an empty native strip before the
input island appeared. The revised contract reveals the complete island over
the existing terminal first, then reserves rows at the shown endpoint; hiding
releases them while the island is covered. The clip excludes empty outer
padding. Output is briefly behind the overlay during the transition, not at rest.
A shallow clip starts with the text baseline visible, while one opacity value
reveals the complete group. The input subtree is uncached only during this
opacity transition because the pinned view-cache key does not include inherited
opacity. Its stable endpoint remains cached.

The follow-up visual pass extends this geometry to split panes and the agent
panel. Each terminal host gets a pane-local rounded mask intersected with the
workspace mask; the shared window transition underlay is deliberately excluded.
Unchanged masks retain their paths, and unsplit panes remove the local mask.
Compact-tab details anchor to measured tiles rather than following the pointer.
Surface labels use UI typography and no longer sit inside an extra nested track.

Further inspection found two local invalidation gaps: host/surface autoresizing
was still enabled despite GPUI owning their frames, and the mask cache trusted
equal bounds without checking its current layer attachment. Disable that second
geometry owner, verify the actual host frame before skipping sync, and rebind
cached paths when a layer is replaced. Linked tests exercise real NSScrollView
frame changes and same-size layer replacement. These establish correctness gaps
in the prototype; live drag acceptance is still required to establish that the
reporter's complete gray-region failure is resolved.

## Verification and remaining acceptance

Focused macOS tests cover safe-area geometry, bounded width, caption clearance,
native material caching/hit testing/hiding, clipping, input, tabs and panes.
The macOS app and CLI build and strict Clippy checks passed. A real isolated
session accepted a new tab and executed a harmless printf through the normal
visible-shell CLI execution flow; its output was read back through the control API.

Preview inspection includes light/dark and expanded/compact navigation. Some
automated window captures lagged the reported UI state and terminal readback;
this limits visual acceptance, not a reason to declare presentation correct.
Rapid mode changes, the new-tab upper-left boundary, terminal scrolling and
material continuity still need user acceptance. No Windows/Linux runtime or
frame-rate benchmark was performed. This work does not close the separate
internal-display/battery native-presentation regression.

The pane follow-up passed 111 focused macOS tests, including linked native
tests for intersecting masks, sibling isolation, cache reuse and mask removal.
The app/CLI build and strict Clippy checks passed. A temporary surface reached
shell-ready state and was removed without closing its pane. However, live
inspection also showed stale GPUI chrome and gray native regions until a window
resize refreshed presentation. Sampling found the main thread in its event
loop, not blocked in the new clipping calls. The cause is not established and
this is not a passed visual acceptance. Keep the prototype local until the
presentation mismatch and light/dark appearance have been confirmed.

The next local build passed 113 focused tests, including actual host-frame
validation, same-size mask detachment/layer replacement and the single-stage
input reveal. Strict Clippy and the app/CLI build passed. In a temporary
three-pane tab, dragging the vertical divider and releasing preserved rounded
edges without gray residuals. That tab was closed and the original three tabs
were preserved. A title-drag attempt did not perform a reorder, so it is not
reported as a passed reorder test. Direct background launch also reproduced a
separate preview limitation: traces showed no new prepaint/paint after API
changes until Finder activated the already-running bundle. Distinguish that
frame-delivery state from a failed GPU submission or the reporter's drag case.
The final preview is activated, but prolonged dragging, dark-theme desktop
inspection and the internal-display/battery scenario still need acceptance.

A subsequent report showed a gray terminal region after a pane was dragged to
the temporary horizontal strip in vertical-tab mode. Code inspection confirmed
that pane drags unnecessarily exposed a second tab destination and undocked
the caption. Vertical mode now keeps its existing sidebar destination and
caption geometry. Pane reorder and promotion explicitly invalidate native
layout for reused surfaces. These close identifiable layout-ownership gaps;
they do not by themselves prove the complete reported gray-region failure is
resolved. Automated title drags on this machine have not performed a reorder,
so the next preview still needs that manual acceptance.

This follow-up passed 118 focused macOS tests, formatting, strict Clippy and an
app/CLI build. The restarted preview was activated through Finder and inspected
with three panes, the narrower command island and the compact-tab hover card.
The original two tabs were retained; the disposable test tab was removed.
Windows/Linux native layout paths and upstream dependencies are unchanged, but
their runtime rendering was not tested. Shared hover/composer styling is part
of this visual change; do not describe those styles as macOS-only.

The next regression fix removes the macOS single-pane early return so every
terminal leaf clears or replaces its local mask during prepaint. A linked GPUI
test seeds an old split clip on a real Ghostty view, renders it through the
single-pane path and checks that the full resolved bounds replace it at radius
zero. A nested-tree geometry test checks divider reservations for both axes.
Rounded drop regions and narrow resize feedback replace the old rectangular
overlays. Search controls and the expanded navigation toolbar are tightened;
small prototype shadows follow the maintainer's explicit request and leave
native terminal opacity unchanged. Runtime acceptance and performance limits
remain separate from these code-level checks.

The first elevation preview exposed another composition mistake: a normal
drop shadow painted by a transparent GPUI wrapper shaded the native surface's
interior because no opaque GPUI fill followed it. The corrected primitive
clips shadow painting to four exterior strips. A geometry test verifies that
those strips do not intersect the content at small, composer and large canvas
sizes. This avoids both the gray interior and full-content shadow fragments.

The corrected follow-up passed 128 focused macOS tests, formatting, strict
Clippy and the app/CLI build. In a disposable two-pane tab, the right pane was
closed through its visible header control; the surviving terminal filled the
canvas without retaining the half-pane mask. The disposable tab was removed
and the original three tabs retained. The final masked-shadow preview was
inspected after activation and no longer shaded the terminal interior. These
are settled-state checks, not high-speed frame capture or a frame-rate
benchmark. Portable drop-bound calculations changed to include dividers;
Windows/Linux runtime acceptance remains unavailable.

## Lessons

Define the canvas, safe area, caption and popover geometry together. Do not call
an inset strip a floating composer or treat a unit-test count as visual approval.
Use small cached corner paths, never a viewport-sized path with a hole. Keep
native material ownership confined to the reserved area; do not solve seams by
making the entire terminal opaque.
