# Linux pointer input drifting from painted columns

**Date**: 2026-10-17
**Issue**: #486

## What happened

On Linux at a fractional display scale, clicks, selections, link hovers and
mouse reports landed on a different cell from the one under the pointer. The
error grew toward the right edge by about one column for every 50. With a 14 px
font at 1.7x, a click in the middle of column 150 selected column 153, and a
full-screen program reading the mouse saw column 154 (1-based). It was found
while fixing #485, the same mismatch in the pseudo-terminal (pty) grid size.

## Root cause

`render()` in `crates/con-app/src/linux_view.rs` places a column every 9 logical
px, which is 15.3 device px at 1.7x. Pointer input used the whole-pixel device
cell, `round(9 × 1.7) = 15` px, in three places:

- `selection_input_from_event_position` divided the device-px pointer position
  by 15 px to find Con's cell.
- The same function gave Ghostty's selection gesture that position and a
  15 px cell width. Ghostty uses them for drag columns and for the half-cell
  rule that decides whether the cell under the pointer is selected.
- `report_mouse` gave Ghostty's mouse encoder the device-px position, and the
  encoder divides by the 15 px cell it was configured with.

The 0.3 px error per column adds up to about 3 columns at column 150. Rows
drift the same way when the row height times the scale is not a whole number,
for example 20 × 4/3 = 26.67 px painted against 27 px used.

## Fix applied

- A pure `pointer_in_grid` maps the pointer onto the painted grid in logical
  px, so the cell matches what `render()` draws.
- It then rescales the position it gives Ghostty so each painted cell spans
  one whole-pixel device cell. Ghostty's own division by the 15 px cell then
  finds the painted cell, at the same point within it.
- The selection gesture gets that position with the padding included. The
  mouse encoder gets it measured from the grid origin, because Con configures
  the encoder with zero padding.
- The autoscroll height goes through the same conversion, so the pointer and
  the pane's bottom edge are compared in one space.

The paint code, the pty size and the cell size reported to programs are
unchanged.

## What we learned

Every path from the screen to the terminal has to use the paint code's
geometry: the pty size (#485), and now pointer input. Unit tests check every
column of a 1000-column grid and every row of a 60-row grid at seven scales.
Tests through a real pty check the mouse report bytes and link lookup at 1.7x.
A change that brings the rounded device cell back into hit-testing fails them.

Windows doesn't have this split. Its renderer hands the shader the whole-pixel
`metrics.cell_width_px` (`crates/con-ghostty/src/windows/render/mod.rs`), and
`windows_view.rs` hit-tests with the same value.
