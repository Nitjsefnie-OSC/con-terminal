# Pane Header Alignment

## What Happened

During the pane-motion design review, the terminal pane title was found to
reserve 20 points on the left but 40 points on the right when both zoom and
close were present. Its apparent center therefore sat left of the pane center.
Inactive titles used solid ink, but focused titles and action icons still used
partially transparent glyphs over the native terminal-adjacent chrome.

## Root Cause

The original symmetric-layout comment accounted for zoom only. Adding close
changed the trailing width without updating the leading spacer. Action buttons
were independent custom divs rather than the same component controls used by
the rest of the window.

## Fix

The title spans the header with equal reserved action space on both sides.
Actions occupy a fixed absolute trailing slot, with vertically centered
component buttons. Hover reveals controls without moving the title. Focus is
expressed through solid ink and a small opaque fill change. The terminal body,
native surface geometry, transparency, and terminal process lifecycle are not
changed.

## Lessons

- Verify control and text bounds together, including narrow panes, focus, zoom
  and both themes. A comment describing symmetry is not a geometry test.
- Keep display decoration separate from PTY/grid geometry. This chrome fix is
  not terminal-body animation or a retained-frame capability.
- Extending the lifetime of a GPU resource is not equivalent to freezing its
  contents. The separate motion study records the per-backend ownership gaps.

## Verification

The 19 pane-tree tests passed, including light/dark geometry and multi-pane
hover/click fixtures. The native macOS app and companion CLI built successfully.
An isolated preview exercised split, zoom/restore, closing a nested pane with
the component control, and immediate terminal input afterward. Light and dark
rendering were inspected. Windows/Linux native runtime validation was not
available; no terminal backend changes or cross-compilation were performed.
