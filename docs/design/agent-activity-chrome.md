# Agent Activity Chrome

The sidebar has three distinct jobs: choose the session's provider/model, show
what is happening now, and retain the conversation and tool trace. Keep those
regions legible without repeating the same status in each one.

The panel meets the titlebar with a slight neutral material step and a quiet
continuous header separator. Its left separator meets that header line, without
independent inset endpoints that resemble a missing corner. Do not clip the terminal
junction into a rounded cutout: that reveals the desktop behind native surfaces.
There is no full-window titlebar rule, outline or opaque transition overlay.

## Live activity

- A submitted user message immediately enters a presentation-only waiting state,
  before context gathering or the first provider event. Show the shared working
  indicator with `Working…`; change to Thinking/Writing as events arrive. Stop,
  completion and errors end the feedback. Restored user messages remain idle.
- Live tools, approval requests and the working indicator belong to the transcript
  tail, not a fixed footer or a second stream below the session controls. Show one
  working indicator while busy; approval is static and does not add a spinner.
- Animate only this summary. Approval is a static shield; it is waiting for the
  user, not work progressing in the background. Trace rows retain their tool
  icons and running/completed states without additional spinners.
- The working indicator shares the horizontal tabs' three pulsing bars and
  1.2-second cycle. Reuse the retained activity renderer rather than starting
  a separate timer or modifying message data. GPUI animation notifications still
  invalidate ancestor views; retain the transcript's parsed/block caches.
  Clipped and hidden markers stop scheduling frames, and reduced
  motion uses static bars. The status label uses the same header columns as
  reasoning and tool calls.
- Use the same compact row geometry for live and saved activity: a readable
  UI-font action label and a truncated mono-font argument preview. Expanded
  output is the reading surface for full details.
- Separate controls, activity, and messages with space and a quiet neutral fill,
  not stacked full-width and inset rules or a tiny uppercase kicker.
- Use solid semantic foreground tokens on transparent glass. Translucency
  belongs to the fill, not the text.

## Session controls

The provider icon identifies the provider; its tooltip names it. The model
selector takes remaining width and shrinks before the action buttons. Stop,
auto-approve, history, and new conversation remain component-library controls.
Do not add an unrelated branding glyph or a second busy indicator to this row.
Both selectors use the same 26-point height and hover-only fill, without an extra
badge inside the provider button. Icon actions use the same height and corner
radius. The model button uses intrinsic width within its
flexible region, keeping its caret beside the label rather than across empty
space. Long names remain available in the tooltip, and icon-only actions have
explicit accessibility labels.

Activity headers use one short title and a compact step count on the same row.
The title truncates before pushing the count outside a narrow panel. Review
status belongs in the title and individual rows, not repeated in the count.

## Performance and behavior

Expanded tab activity uses three center-anchored, staggered bars in the existing
retained activity layer. Compact tabs keep their brand icon and ring; numeric
progress stays determinate. Reduced motion uses static bars, and invisible
markers do not request animation frames. Activity never changes the tab's width.

## Transcript hierarchy

Read a completed turn in order: identity, grouped activity, final reply.
Reasoning and tools share one activity disclosure rather than two competing
section headers. Completed groups summarize elapsed work; individual executable
rows pair a UI-font action with a truncated mono command preview on one line.
Only an expanded result owns a content surface.
The answer remains the primary reading surface. Reasoning is a plain disclosure,
without a brain glyph or word count. Expanded reasoning must not be silently
clipped. Activity uses a quiet heading rather than a filled enclosing card.
Routing notes are secondary text rows; executable steps have compact action
headers. Output has one inset surface, never a surface inside a surface inside
an activity card. Set the header's own radius: parent overflow clipping does
not round a child's hover fill in GPUI.

Disclosure controls use component buttons where possible, including keyboard
activation and accessible action labels. Opening tool output must not restart
an opacity-from-zero entrance animation on unrelated redraws.

Long replies retain parsed data and per-block entities. An explicit Continue
reading action hydrates all remaining blocks in bounded batches, shows progress,
and leaves Show less available after completion. Collapsing must stop pending
manual hydration; streaming retains its existing automatic hydration path.

Message metadata uses plain UI-font text rather than another filled badge.
Tool names, disclosure labels and message timing use the UI font; commands,
arguments retain the mono font; durations use the UI font. Body text, reasoning and
activity share one left rail, rather than accumulating different nested indents.
User messages use one neutral rounded fill without a decorative corner cutout.

Reasoning, saved tools and live tools share a 32-point header, 10-point side
insets, a 16-point icon column and an 8-point column gap, scaled with UI text.
Their labels share one baseline and left rail. Expanded content begins beneath
that label column. Resting headers have no fill; hover and keyboard focus use
the component button's rounded feedback. Do not turn one tool header into a
filled tile while adjacent reasoning remains a small text button.
Code fences have one rounded surface, a quiet language label and copy action;
the scrollable code body does not have a second fill or outline. Keep the existing
highlight cache and scroll extent. The composer retains its mono input font,
with balanced horizontal padding and an inset that follows the transcript gutter.
The composer reserves its intrinsic height in the panel's flex layout. It does
not cover the transcript with absolute positioning or depend on fixed bottom
padding to hide that overlap. The transcript viewport ends above the composer.

The foreground event dispatcher consumes at most 64 agent events per session
per turn, in FIFO order, and awaits between turns even while work is available.
This prevents a continuously ready producer from monopolizing native input
dispatch. It is not a global time budget for every workspace queue.

The live tail is one transient row in the same virtualized list as messages.
Only that row is remeasured for live tool/approval changes; retained Markdown
entities and parsed data remain unchanged. Replace the old tail slot when a new
assistant message occupies its index, even when the list's total count does not
change. Otherwise an activity row's cached size can survive as a reply's size.
Tool and approval content have intrinsic height and scroll with the conversation,
without an independent 320px footer scroller. Approval callbacks, tool results,
streaming and composer focus retain their existing execution paths. Follow-output
behavior still respects manual scrolling and exposes the existing latest action.

Layout tests cover a 280px panel, a wide panel, enlarged UI text, and CJK/long
tool details. Native visual acceptance should cover thinking, writing, tool
execution, approval, and idle in light and dark themes.
