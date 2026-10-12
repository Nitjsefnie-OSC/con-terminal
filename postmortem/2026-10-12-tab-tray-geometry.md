# Uneven Insets and Icon Centres in the Tab Tray

## What Happened

During the horizontal-tab redesign, the selected surface looked cramped
vertically and loosely inset horizontally. A later review also identified
uneven space around button icons. The initial tests checked target bounds and
click behavior, but not the painted layers or the contents of those targets.

## Root Causes

The tray was inset vertically by one point. Targets had four points of
horizontal padding, while the selected layer had two points of vertical
padding. These independent rules produced one point of vertical clearance
and four points of horizontal clearance. The user's Retina screenshot
confirmed two-pixel top/bottom gaps and an eight-pixel leading gap.

Scaled icons also had fractional dimensions. A new layout assertion reproduced
a 17.5-point icon at origin `(2, 5)` inside a 22-point button at `(0, 3)`.
Their centres differed by 0.25 points in both axes. A centred outer button
therefore did not establish that its SVG was centred after pixel snapping.

A follow-up screenshot exposed two gaps in that verification. The identity
row needed to fill its target's content width, but native inspection confirmed
the extra trailing clearance came from a twelve-point reorder drop target
included in the painted tray. The close overlay was a block container; its inline
button followed text-baseline placement. A circle could be centred within its
button while the whole button sat below the tab centre.

The next visual review also exposed a mismatch between icon-row padding and
the capsule radius: a ten-point leading inset made the fourteen-point close
circle sit thirteen points from the left edge but three points from top/bottom
on macOS. Vertical centring alone did not make the nested circles concentric.

## Fix

The painted tray now shares the target height. One two-point selection inset
controls horizontal target padding and vertical selection-layer padding.
Production and tests use the same tray and selection-layer helpers.

Titlebar icons use even logical dimensions after UI scaling, keeping them on
the same grid as their square controls. The close hit target is explicitly
square, and its visual circle stays separate from that target. No per-icon
position offsets or changes to terminal rendering are needed.

The identity row now fills available content width. A shared centred flex
overlay places the close button independently of the inherited text baseline.
The trailing drop target sits immediately outside the tray as an absolute
hit region; it retains end-of-strip reordering without enlarging the fill.
The hover interaction test checks the circle against the tab centre, and the
inset test checks each painted layer against its own target as well as the tray.

The identity-slot centre now matches the capsule's left arc centre. Leading
padding derives from the surface and hit-target heights, rather than an
independent text inset. The hover test checks equal left/top/bottom clearance
against the painted surface. Attention dots use trailing space instead of
inserting another flex item before the identity slot.

## Verification

Regression tests check selected surfaces at both tray ends at wide and narrow
widths, and the tray's placement within the titlebar. Button tests check square
SVG bounds, glyph and circle centres, focus preservation, and click handling
with default and larger UI font sizes. These tests are included in native
Windows/Linux CI; local native inspection is on macOS.

## Lessons

Nested surfaces need one geometric rule, not independently tuned offsets.
Outer bounds and successful clicks do not establish visual alignment. Check
painted layers and inner content at more than the default UI scale, then
inspect the native result.
