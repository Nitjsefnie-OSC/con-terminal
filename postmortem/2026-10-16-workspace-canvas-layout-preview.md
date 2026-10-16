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
- Changing to horizontal tabs retained the outgoing caption spring, even though
  horizontal tabs immediately needed the complete global row.
- A GPUI-colored safe-area extension showed a material step against the native
  terminal despite equal RGB and opacity inputs. The underlying color/compositing
  mechanism has not been established; equal parameters are not visual proof.

## Changes

The canvas now owns the terminal and its bottom safe area. The composer is a
centered overlay within that safe area, capped at 960pt, with terminal rows
reserved so output cannot hide underneath. Its popovers share its horizontal
geometry. A cached passive native material view supplies only the safe-area
background; it neither receives input nor covers terminal output. Native
presentation synchronization also covers changes to this view.

The macOS canvas gutter is 4pt with 8pt corners. Sidebar rules are removed,
the visibility control remains available, and horizontal mode immediately
restores the caption row. Windows/Linux retain their previous layout paths.

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

## Lessons

Define the canvas, safe area, caption and popover geometry together. Do not call
an inset strip a floating composer or treat a unit-test count as visual approval.
Use small cached corner paths, never a viewport-sized path with a hole. Keep
native material ownership confined to the reserved area; do not solve seams by
making the entire terminal opaque.
