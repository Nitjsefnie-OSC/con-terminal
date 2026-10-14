# Chat Disclosure Presentation

## Report

The maintainer reported rectangular hover fills inside rounded tool cards,
excessive nested output surfaces and apparently missing content after clicking
Show more. The screenshots also showed decorative brain glyphs and word counts
competing with the actual reply.

## Grounded findings

Output used an outer trace fill, a step fill, a result wrapper fill and an inner
result fill. The hover header itself had no radius; its parent's overflow clip
did not give its background rounded corners. Reasoning was capped at 200 pixels
with overflow hidden. Disclosure entrance animations started at zero opacity
and were constructed on each render.

The original expansion path passes a focused retained-view click test for a
15-block reply. That does not reproduce or explain every part of the reported
blank state. Do not describe the screenshot as proof that message data was lost.
Its control did disappear after the final batch, providing no persistent feedback
or way to collapse the expanded reply.

## Change and acceptance

Use answer-first typography, a quiet activity group, compact tool rows and one
output surface. Remove brain glyphs and word-count chrome. Give interactive
headers their own corner radius and stop replaying disclosure entrance effects.
Use a component button for reading more, with bounded asynchronous hydration,
explicit feedback and a retained collapse action. Preserve streaming's existing
automatic hydration and the parsed-data/per-block cache boundaries.

Acceptance must include real clicks through the virtualized list, growing row
geometry, expansion/collapse, light/dark and narrow layouts, restored content and
streaming. The exact reported blank state still needs native reproduction before
claiming its sole cause is identified. No harness, provider protocol or terminal
backend changes belong in this visual redesign.

## Verification

All 33 agent-panel, 9 activity and 37 Markdown tests pass, including real disclosure clicks and row-height
growth in the virtualized list, collapse, automatic streaming hydration, narrow
headers and scaled controls. The linked macOS app and companion CLI build,
Clippy with warnings denied, and workspace formatting checks pass.

An isolated native preview was inspected in Flexoki Light and Dark, at normal
and 280-point panel widths. Activity and tool details expand; a truncated tool
output expands to its remaining lines and retains its collapse control. The
fixture uses restored conversation data without contacting an LLM provider or
changing the maintainer's configuration.

Windows/Linux native UI, live-provider streaming and the exact reported complex
conversation remain outside this verification. The streaming test exercises the
panel's actual update/parser/hydration path, not a live provider response.

The follow-up visual pass aligns reply sections on one rail, removes nested code
fence fills, unifies session-control geometry and balances composer padding.
The preview now reads the maintainer's normal configuration and provider
credentials, while retaining separate test session, history, conversation and
socket paths. The configured provider menu was checked; no live LLM request was
sent by the agent during visual verification.

## Transcript layout follow-up

The previous layout appended completed activity after the final reply, but put
live tools and approvals in a separate fixed footer. This split reading order
was retained during the first visual pass and was insufficient. Both now use
the conversation's virtualized list; completed activity precedes the final reply.
The transient live row is replaced when an assistant message takes its index,
including equal-total-count transitions. Its layout is remeasured separately
from retained message Markdown. The panel's top and left separators also had
independent 12px endpoint insets, creating the reported missing-corner appearance;
they now meet continuously without changing native terminal geometry or opacity.

Reasoning and tool history now share a single activity disclosure. Its completion
and restoration rules also collapse reasoning-only groups; the former condition
only recognized groups containing tool steps. A real-click test covers expanding
such a group and its reasoning without hiding the answer. Composer geometry is
now intrinsic flex layout rather than an overlay. Narrow/scaled composer tests
assert that the transcript viewport ends above the input surface.

Native inspection of the maintainer's restored conversation confirmed the compact
work summary, reasoning and command-row disclosures, continuous panel junction,
and reserved composer space. This is not native Windows/Linux acceptance or a
live-provider streaming benchmark. The last contrast pass uses solid secondary
text for reasoning rather than multiplying its opacity on panel glass.

The subsequent row pass removes the inconsistent short reasoning button beside
a full-width filled tool header. Reasoning, saved tools and live tools now use
one header layout, including icon slots, label baselines and trailing controls.
A geometry regression test checks identical left rails at 280/640-point widths
and enlarged UI text, allowing subpixel center differences from pixel rounding.
Native light-theme inspection of the configured conversation covers resting,
hovered and expanded reasoning/tool rows. This last pass has not received native
dark-theme or Windows/Linux visual acceptance.

The working indicator now shares horizontal tabs' three-bar renderer and timing,
instead of a separate spinning glyph. Status-row tests cover Thinking/Writing
alignment and removal when idle. Activity tests cover frame scheduling,
visibility and reduced motion. The performance experiment also established that
GPUI animation notifications invalidate ancestor views, even across a cached
panel boundary. A retained animation entity is not proof of zero parent renders;
the existing Markdown data/block caches remain necessary. The standalone marker
test checks one frame callback per tick after initial visibility notifications,
not an unsupported promise that ancestors never redraw. Native inspection covered
the restored light-theme conversation; live indicator motion still needs the
maintainer's interaction test, and no live LLM call was issued by the agent.

## Submission feedback

Adding a user message unconditionally set the panel to Idle. The harness gathers
terminal context asynchronously before the provider emits its first event, so
the accepted request could have no visible working feedback during that gap.
New user messages now enter AwaitingResponse synchronously, displaying `Working…`
in the same status row. Reasoning/token events change the label without adding
another indicator. Stop, error and completion clear it. Restore uses its separate
idle path, so old user messages do not masquerade as pending requests. The change
also covers edited resubmissions and background-tab requests through shared panel
state. A rendered-panel test verifies feedback before any harness event, the
first-token transition, and removal on Stop; state tests cover errors, completion
and restore. No synthetic model message or harness event is introduced.
