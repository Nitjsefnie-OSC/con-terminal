# Pane Motion Must Follow the Split Tree

## What Happened

During implementation of pane body transitions, single split, close and zoom
tests passed. A consecutive close of nested panes did not: independently
interpolated leaf rectangles left a region uncovered while their parent grew.
The failing prototype was caught by a geometry regression test before commit.

## Root Cause

A pane rectangle is not independent of its ancestors. Interpolating each leaf
between old and new bounds can work for one topology change, but breaks when
an inner transition continues while its parent retargets. Filling the hole
with an opaque cover would mask the symptom and change terminal transparency.

## Fix

Animate a presentation-only split tree. Each split computes its two children
and divider from a shared parent rectangle using animated ratios and gap sizes.
Retarget unchanged inner motions without restarting their clocks. Prune
settled dead branches independently of other active transitions.

Terminal bodies keep their final pixel dimensions while their viewports move
and crop. The numeric coordinator retains no terminal, process or screenshot.
Only uncovered margins use the original-alpha terminal background. macOS
native viewport changes use the existing frame-scoped presentation mechanism;
pure movement does not force a synchronous Ghostty draw every frame.

## Verification

- Native drag checks exposed a second error: a divider child already parented
  in Taffy was re-rooted for presentation. Its hitbox moved to 999.5 points
  instead of the visible divider's 500 points in the GPUI fixture. Give the
  wrapper its own layout node and keep the presented child independent. The
  regression assertion now compares hitbox and visual positions, not only
  whether a click at the reported hitbox location invokes its callback.
- Geometry tests assert total coverage and pairwise non-overlap for both split
  directions, nested close, zoom reversal and interrupted consecutive closes.
- A real GPUI fixture checks unchanged body pixel size across intermediate
  frames. Fractional-scale tests cover 1, 1.25, 1.5 and 2x.
- A macOS PTY probe observed one SIGWINCH for each of split, zoom, restore and
  closing the adjacent pane, rather than one per animation frame.
- Windows/Linux native runtime, IME and GPU behavior require their native CI
  and real-machine validation. They were not cross-compiled or measured here.

## Lessons

Test interrupted hierarchy changes, not only isolated endpoint transitions.
Keep visual geometry separate from terminal grid geometry. Do not equate a
retained live GPU surface with an immutable snapshot, or claim a frame rate
without measuring it.
