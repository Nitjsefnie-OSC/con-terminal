# Compact Horizontal Tabs

Horizontal tabs are window navigation, not terminal content. They use the
configured UI font, while terminal text and code keep their monospace fonts.

## Layout

- Keep the existing titlebar heights: 28 points on macOS and 36 elsewhere.
- Keep tab hit targets at their existing height. Use a twenty-point close control.
- Center tab hit targets vertically, with matching space above and below.
  A continuous, lightly tinted tray groups the targets. It has no border or
  shadow and shares their height: 24 points on macOS, 30 elsewhere.
  Center the tray in the titlebar, leaving two points above/below on macOS
  and three elsewhere.
- Size targets to their contents, capped at 220 points per tab. Short names do
  not leave a long empty tail; long names receive more room. Targets shrink
  when window space runs out, and names truncate after the fixed identity slot.
  Unused titlebar space remains available for window dragging.
  Inline renaming temporarily uses the capped width so its full-width editor
  cannot collapse inside an intrinsically sized target.
  Keep the twelve-point trailing reorder hit region outside the painted tray.
  It must not add blank width to the last tab's visible end.
- Align the identity-slot centre with the capsule's left arc centre. Derive
  the leading inset from half the difference between surface and slot heights:
  zero points on macOS, three elsewhere. The fixed twenty-point identity slot,
  six-point icon-to-label gap, and twelve-point trailing inset separate the
  icon and text. Two-point
  target padding separates adjacent groups without separator rules.
  The selected surface wraps the icon and name as one unit. Use the same
  `TAB_SELECTION_INSET` for target horizontal padding and selection-layer
  vertical padding. Selection at either end of the tray then has two-point
  clearance on all four sides, not separately chosen horizontal/vertical gaps.
  Its height is 20 points on macOS, 26 elsewhere. Both surfaces use half-height
  corner radii. Attention marks keep their semantic color.
  The identity row fills the target's content width, including after a long
  label truncates. Otherwise unused width can become an oversized trailing gap.
- Share one fixed twenty-point leading slot between the identity icon and close
  control. Hovering the tab hides its icon and reveals the close control in
  exactly the same position. The capsule never moves, and the right edge has
  no reserved action slot. Attention marks remain visible while hovered.
  Use a twenty-point component button with a fourteen-point circular fill,
  centered vertically in the identity slot. The visual circle is smaller than
  the hit target and the name capsule.
  Its clearance from the capsule's left, top, and bottom edges is equal:
  three points on macOS, six elsewhere. Check against the painted capsule,
  not the outer hit target. Attention dots occupy the trailing inset without
  shifting the identity slot or label.
  The Phosphor thin close glyph uses a ten-point square at default UI size and
  scales in even-point steps. It stays subordinate to the identity
  icon but remains readable. A quiet circular fill is visible whenever the
  close control appears, then strengthens when the control itself is hovered.
  Keep the button explicitly square and check the inner circle and SVG centres,
  not only the button's outer bounds, including with a larger UI font size.
  The overlay slot is also a centred flex container: an inline button in a
  block container follows the inherited text baseline rather than its centre.
  Check the circle against the actual tab, not just against the button.
  Its icon becomes clearer on hover, without a destructive red treatment.
  The close control appears only while its tab is hovered, including the
  selected tab. Empty space still activates and drags the tab.
- New-tab drag placeholders use a 120-point target. Reordering uses measured
  target bounds rather than assuming equal widths. Reorder, rename,
  middle-click close, context menus, and native caption hit testing retain
  their existing handlers.

## States

The tray tints the actual translucent chrome; resting tabs do not each receive
their own fill. The selected surface is lighter than the tray in both themes:
a slight lightness lift in light themes, a foreground mix in dark themes.
Its title uses medium weight. Hover adds a restrained fill around the whole
identity group, never a separate name badge or rectangular outer frame.
Labels use solid theme-derived ink at normal weight. Selection must not depend
on making unselected titles hard to read.
Identity glyphs use a fourteen-point frame at default UI size, leaving room
beside the capsule's left arc. The six-point gap groups the icon and label
without adding padding inside the label itself. Larger UI sizes cap the frame at the
twenty-point identity slot; the slot centre and label position stay fixed.

User-assigned accent fills and configured inactive accent opacity remain
supported. Activity and attention indicators retain their semantic colors.
Labels do not inherit glyph opacity from these fills. Icons use solid,
theme-derived ink and the existing Phosphor assets.

Names come from the same retained presentation as the sidebar, including user
renames. Do not shorten by character count: the label truncates according to
its actual available width. The tab tooltip includes the full name and existing
subtitle/status when distinct. No path lookup, terminal query, or generated
summary is performed while rendering. The close button has its own label and
tooltip, and consumes mouse-down without moving terminal focus or dragging the
window.
Do not occlude the parent tab's hover hitbox: a hover-only close control would
hide itself as the pointer enters it. Consume mouse-down to keep activation
separate, and test hover followed by a click, not just the button in isolation.

The shared tray ends with the tab contents, not at the window's action buttons.
It must not consume the remaining draggable titlebar space. No nested name
capsules, divider rules, borders, or shadows are added to this navigation group.

## Dragging

The floating horizontal-tab preview uses the measured source surface width,
the same capsule height/radius, identity slot, UI font, and solid ink as resting
tabs. Pointer offset is relative to the painted surface, not its padded target.
The preview does not retain the older fixed-width rectangular card.

One workspace-root drag-move handler owns horizontal reordering. It probes the
floating capsule's centre and compares it with the actual prepainted neighbour
bounds, excluding the dragged tab's own placeholder. Convert the resulting
insertion position back to the original tab order before committing a drop.
The source's visual index belongs to the layout snapshot, not a newly requested
drop slot that may not have painted yet. Keep the last valid target through
missing or partial snapshots; do not reset or reorder based on incomplete data.

Pane-to-tab promotion retains its separate ghost/drop flow. Resting mouse moves
and pane-resize handling do not run horizontal reorder calculations.

## Scope

Titlebar actions retain their twenty-two-point hit targets and use fourteen-point
Phosphor icons at default UI size, scaling in even-point steps. Fractional icon
dimensions can snap their origin and size differently and move their centre.
Open-panel states have a persistent neutral fill and solid
foreground icons instead of accent-colored glyphs. Hover and press strengthen
the same fill, with six-point rounding and centered placement. Accessibility
toggle metadata and shortcut tooltips remain unchanged.

This change is paint and layout only. It adds no timers, theme parsing,
terminal queries, native-surface changes, or new animation. Window glass,
presentation synchronization, single-tab chrome, and vertical tabs are not
changed.

## Acceptance

Inspect dark and light themes, assigned tab colors, hover and selected states,
long titles, increased UI font size, narrow windows, and drag placeholders.
Exercise click and keyboard activation, close, rename, and reorder. Native
macOS verification does not substitute for Windows/Linux visual acceptance.
