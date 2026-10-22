# File preview panes bypassed the workspace card material

## What happened

Opening a file beside a macOS terminal produced a square editor pane without
the terminal card's inset, corner treatment or fine edge. Images and Markdown
used the same inconsistent container.

## Root cause

`PaneTree::render_leaf` returned early for editor content, before the terminal
card wrapper. The editor root and status row also painted rectangular fills.
Its header asked the native terminal canvas which corners to round, although
an editor pane is GPUI content rather than a native terminal host.

The first local preview reused `PaneSurround` as well as a rounded GPUI fill.
That was incorrect: the native surface needs a complementary corner raster,
but GPUI already antialiases its quad. Independently covering the same curve
twice produced a visible rim in the reporter's dark-theme screenshot.

## Fix

Use the existing pane geometry and concentric radii for the editor branch.
One GPUI quad paints the fill and edge over a flat chrome backing, without
native corner cutouts or a second antialiased outline. One fill owns the four corners;
the header, empty state, root and status row do not repaint square backgrounds.
The body remains between the header and status row. Native terminal clipping,
glass, presentation synchronization and Windows/Linux fills are unchanged.

The bottom input tray now uses the same single-quad material. Its fill and
edge previously lived in separate branches, with independently multiplied
reveal/depth opacity. Both now fade inside the existing reveal container,
with unchanged terminal reservation, input focus and view-cache lifetime.

## Lessons

Pane-level appearance belongs to the workspace renderer, not the kind of file
being displayed. Sharing geometry does not mean sharing native compositing
workarounds. Rounded backgrounds alone do not clip GPUI descendants.
Cover empty and populated panes, clean single-pane mode and intermediate canvas
depths when testing a new pane content type.

## Verification

- Final macOS linked app suite: 682 passed; focused editor suite: 149 passed;
  frame/material suite: 22 passed. All 27 live `con-test` cases passed.
- Strict binary Clippy, Rust formatting, diff checks and docs manifest passed.
- Added a rendered geometry regression covering empty/populated/closed file
  states at canvas depths 0, 0.5 and 1, plus component-control focus and
  single-quad material tests at 1x, 1.5x and 2x.
- Independently configured macOS light/dark previews exercised text, image,
  Markdown, file-tab controls, pane zoom and input hide/show. No user settings
  or production sessions were used for these checks.
- No Windows/Linux hardware was available. Their fills remain cfg-isolated;
  these macOS checks do not establish native Windows/Linux acceptance.
