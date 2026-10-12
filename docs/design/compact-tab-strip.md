# Compact Horizontal Tabs

Horizontal tabs are window navigation, not terminal content. They use the
configured UI font, while terminal text and code keep their monospace fonts.

## Layout

- Keep the existing titlebar heights: 28 points on macOS and 36 elsewhere.
- Keep tab hit targets at their existing height. Use a twenty-point close control.
- Center tab hit targets vertically, with matching space above and below.
  A continuous, lightly tinted tray groups the targets. It has no border or
  shadow, and its painted height is two points smaller than the hit targets.
- Size targets to their contents, capped at 220 points per tab. Short names do
  not leave a long empty tail; long names receive more room. Targets shrink
  when window space runs out, and names truncate after the fixed identity slot.
  Unused titlebar space remains available for window dragging.
  Inline renaming temporarily uses the capped width so its full-width editor
  cannot collapse inside an intrinsically sized target.
- Align each identity group to the same left inset. A ten-point leading inset,
  fixed twenty-point identity slot, eight-point icon-to-label gap, and
  twelve-point trailing inset balance the visible icon and text. Four-point
  target padding separates adjacent groups without separator rules.
  The selected surface wraps the icon and name as one unit. Its twenty-point
  height leaves even clearance inside the twenty-two-point tray. Both surfaces
  use half-height corner radii. Attention marks keep their semantic color.
- Share one fixed twenty-point leading slot between the identity icon and close
  control. Hovering the tab hides its icon and reveals the close control in
  exactly the same position. The capsule never moves, and the right edge has
  no reserved action slot. Attention marks remain visible while hovered.
  Use a twenty-point component button with a fourteen-point circular fill,
  centered vertically in the identity slot. The visual circle is smaller than
  the hit target and the name capsule.
  The nine-point Phosphor thin close glyph stays subordinate to the identity
  icon but remains readable. A quiet circular fill is visible whenever the
  close control appears, then strengthens when the control itself is hovered.
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

## Scope

Titlebar actions retain their twenty-two-point hit targets and use thirteen-point
Phosphor icons. Open-panel states have a persistent neutral fill and solid
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
