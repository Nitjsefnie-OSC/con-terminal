# Workspace Canvas

## Visual hierarchy

On macOS, workspaces with sidebars or multiple tabs use an inset terminal
canvas. It has 6pt of breathing room and 9pt outer corners. Split panes remain
part of that one canvas: interior dividers are straight, and no separate cards or
shadows compete with terminal content.

The input bar occupies its own layout slot below the canvas. An 8pt inset and
11pt corner radius give it a distinct silhouette without placing a floating
overlay over the last terminal row. It uses Window Chrome, not Terminal Glass.
The terminal keeps its existing transparency, blur and theme. Light chrome uses
a lighter input surface rather than a recessed grey well; dark chrome uses a
quiet foreground lift. Neither needs a shadow or outline.

One tab with both sidebars hidden returns to the original clean terminal:
no canvas inset, no corner mask and no input-island margin. These dimensions
follow sidebar motion continuously rather than snapping when a boolean changes.

Expanded vertical navigation owns the macOS caption controls. The existing
toolbar moves into its top row while the global caption row gives its height
back to terminal content, in one coordinated motion. Its material is supplied
by the sidebar, not by a second translucent header rectangle.
Compact navigation, hidden navigation, Files/Search and horizontal tabs retain
the global row. Native traffic lights must never overlap a 44pt rail or a
terminal pane.

## Ownership and presentation

- GPUI determines the canvas rectangle before terminal children prepaint.
- GPUI paints four narrow gutter rectangles and four small corner cutouts,
  cached by bounds, depth and backing scale. Do not replace this with
  a full-area fill or a window-sized path: GPUI's path compositing cost follows
  the path's bounds even if it has a hole. The caption row owns its own material,
  so docking controls does not double-composite the sidebar background.
- AppKit clips terminal host layers against that shared rounded rectangle.
  Rounding a GPUI wrapper alone does not clip its native sibling views.
- Corner cutouts extend one physical pixel inward over the native mask edge.
  Exact complementary antialiased shapes do not produce full coverage under
  source-over compositing. This small overlap is not an outline or an opaque
  terminal backing; the rest of the terminal stays untouched. Leave AppKit's
  parent view/layer hierarchy alone: Con owns host masks, not the container's
  backing-store composition. No extra native material views or layers are used.
- The clip path is cached per host. A window-scoped geometry generation
  invalidates otherwise unchanged terminal layouts when the outer canvas moves.
- Pane headers round only the corners that touch the canvas's top corners.
  Interior pane headers keep square joins.
- Existing frame-scoped native presentation synchronization remains in place.
  A shape mask must not introduce an opaque terminal backing, a second glass
  fill or a delayed geometry update.
- Caption motion has one owner. Reserved global-row height is the complement
  of docking progress; controls and terminal space move together. Never hold
  outgoing sidebar width to introduce a second phase. Reduce Motion resolves
  directly to a valid endpoint.
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
