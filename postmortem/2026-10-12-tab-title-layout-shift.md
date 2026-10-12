# Tab Titles Moved Navigation Targets

## What Happened

AI-generated tab names changed horizontal tab widths. Neighbouring tabs and
their close controls moved even though the user had not opened or closed a tab.

## Root Cause

The compact strip used intrinsic text widths with a maximum cap. That made
navigation geometry depend on content that changes asynchronously.

## Fix

Tabs divide available width equally using zero-basis flex allocation. Titles
truncate within that allocation, and inline rename keeps the same width.
A 140-point minimum keeps targets usable; overflow uses a horizontal viewport.
Selection changes reveal the active tab without fighting manual scrolling on
ordinary repaints. Drag previews continue to use measured source geometry.
Each equal-width target contains one full-width capsule. The identity icon
stays at a fixed left inset with its label immediately beside it. A separate
close control stays at the right capsule edge, so hover and asynchronous renaming
cannot shift the identity or title region. Drag offsets use the capsule's inset
origin rather than the outer target.
One quiet outer capsule groups the strip. Resting inner tabs remain transparent;
selection and hover use inner capsules, avoiding the heavier appearance of a
row of independently filled buttons.

## Lessons

Asynchronous labels may change content, not navigation target positions.
Regression tests must change labels as well as window dimensions, and verify
both normal equal allocation and narrow-window minimum widths.
Geometrically equal spacing still needs visible grouping. Keep selection,
grouping and geometry independent instead of moving icons to compensate for
changes in the surrounding paint.

Overflow reveal requests are keyed to the current identity, position, count and
laid-out viewport width. Pending requests must yield to newer selections,
manual scrolling and active drags; ordinary repaints must preserve the offset.
Tests deliver actual wheel events and explicit test frames rather than assuming
that refreshing a test window delivers queued platform callbacks.
