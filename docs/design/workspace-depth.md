# Workspace Canvas

## Visual hierarchy

On macOS the terminal is an inset canvas, not another strip of window chrome.
It has 6pt of breathing room and 9pt outer corners. Split panes remain part of
that one canvas: interior dividers are straight, and no separate cards or
shadows compete with terminal content.

The input bar occupies its own layout slot below the canvas. An 8pt inset and
11pt corner radius give it a distinct silhouette without placing a floating
overlay over the last terminal row. It uses Window Chrome, not Terminal Glass.
The terminal keeps its existing transparency, blur and theme.

Expanded vertical navigation owns the macOS caption controls. Once the sidebar
has reached its full width, the existing toolbar moves into its top row; only
then does the global caption row give its height back to terminal content.
Compact navigation, hidden navigation, Files/Search and horizontal tabs retain
the global row. Native traffic lights must never overlap a 44pt rail or a
terminal pane.

## Ownership and presentation

- GPUI determines the canvas rectangle before terminal children prepaint.
- Cached geometry paints only four narrow gutter rectangles and four small
  corner cutouts, leaving terminal pixels untouched. Do not replace this with
  a full-area fill or a window-sized path: GPUI's path compositing cost follows
  the path's bounds even if it has a hole. The caption row owns its own material,
  so docking controls does not double-composite the sidebar background.
- AppKit clips terminal host layers against that shared rounded rectangle.
  Rounding a GPUI wrapper alone does not clip its native sibling views.
- The clip path is cached per host. A window-scoped geometry generation
  invalidates otherwise unchanged terminal layouts when the outer canvas moves.
- Pane headers round only the corners that touch the canvas's top corners.
  Interior pane headers keep square joins.
- Existing frame-scoped native presentation synchronization remains in place.
  A shape mask must not introduce an opaque terminal backing, a second glass
  fill or a delayed geometry update.
- Caption motion has one owner. It restores the global row before moving its
  controls out of the sidebar, and holds outgoing sidebar width until that
  handoff is complete. Reduce Motion resolves directly to a valid endpoint.
  Dragging a pane into a new tab restores the global drop row immediately,
  without making an interaction wait for decorative motion.
- Pane resizing uses measured canvas bounds, including its inset, rather than
  estimating terminal geometry from window dimensions.

## Scope and acceptance

This iteration is macOS-only. Windows and Linux retain their existing caption,
terminal reservation and input-bar layout. A portable implementation needs its
own renderer clipping and caption acceptance; macOS layer masks are not such
an implementation.

Before committing, inspect light and dark themes, horizontal tabs, compact and
expanded vertical navigation, Files/Search, split panes and the input bar's
three modes. Exercise collapse/reveal reversal, pane divider dragging, native
window dragging, focus recovery, scrolling and narrow windows. Terminal output
and selections must stay inside the canvas, and the input bar must not cover
the final terminal row.

This visual change is not a resolution of the intermittent native presentation
seam. Retain the display/power-state acceptance matrix in
[Sidebar Motion](sidebar-motion.md), including the internal-display/battery
reproduction. External-display checks alone cannot close that regression.
