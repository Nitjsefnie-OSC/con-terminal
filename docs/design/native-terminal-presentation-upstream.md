# Native Presentation Upstream Handoff

This is a preparation record, not a submitted PR or a maintainer-approved API.
The technical outline below is AI-assisted. A human author must review both the
code and the communication before submission, following Zed's current policy.

## Scope

Propose one opt-in, frame-scoped GPUI facility for applications that embed
native views beneath the Metal scene. It should let native geometry changes
and the corresponding drawable appear together, without synchronizing ordinary
text output or unrelated GPUI frames. Do not include alpha-blending changes:
those already landed independently in Zed #65571.

The existing discussion is [Zed #65433](https://github.com/zed-industries/zed/discussions/65433).
On 2026-10-18 it was open without maintainer replies. Zed asks contributors to
discuss features before implementing them; confirm the API direction there
before treating the source candidate as an invited PR.

## Where the Reproduction Belongs

The observed failure is in Con, which embeds native Ghostty views. We have not
demonstrated the same failure in the Zed editor. Do not describe it as a Zed
editor rendering bug or manufacture editor before/after screenshots.

Zed's contribution guide, checked on 2026-10-18, requires building and running
Zed, manual testing, and screenshots or video for visual changes. It does not
say the visual artifact must show the editor. Supply two separate kinds of
evidence and let maintainers decide whether they also want a standalone GPUI
example:

- **Embedding reproduction:** matched Con before/after recordings demonstrate
  the native-view problem and the proposed API's effect.
- **Zed regression smoke:** a locally built Zed exercises ordinary editor
  behavior with the API present but unused. This cannot demonstrate that the
  embedding problem is fixed.

## Matched Before/After Capture

Use an isolated Con configuration and disposable terminal fixture. Do not
record credentials, a live agent conversation, or the user's session.

1. Build two otherwise identical embedding candidates against the same GPUI
   revision: one without application synchronization requests and one with
   them. If using a diagnostic request-disable flag, keep it isolated from
   production and disclose it. Record both commit IDs and the exact difference.
2. Use the same window size, nested pane layout, theme, opacity, blur, desktop
   background and display. Keep all geometry and clipping fixes identical.
3. Rapidly show/hide left navigation, the agent panel and the input bar; then
   split, close, zoom and switch tabs. Include reversals during transitions.
4. Capture actual motion, not just settled screenshots. Extract a labeled
   failure frame from the before recording and a corresponding after frame;
   retain the full clips so a frame cannot be mistaken for a frequency claim.
5. Repeat on the affected built-in display on battery as well as the external
   display on AC. Record display scale, refresh rate, macOS version, power mode,
   capture rate and number of trials. Do not infer compositor FPS from a
   lower-rate screen recording.

The original reporter image in Discussion #65433 is historical context, not
a matched after artifact. The accepted October 18 corner preview is also not
evidence for this API: that change concerns static decorative edge coverage.

## Code and Test Package

The existing reviewed candidate is
[`8bd2ec8cd022e3b4c2a0d62e9b39b4c0f1508aa1`](https://github.com/wey-gu/zed/commit/8bd2ec8cd022e3b4c2a0d62e9b39b4c0f1508aa1),
based on official Zed `f16f9652ec57bf806e65b2a0d51bb92a63644914`.
It is not the compatible package snapshot Con consumes. Rebase onto the agreed
official base and rerun checks before submission; old results are historical,
not verification of a new revision.

Keep these semantics explicit:

- Requests coalesce without scheduling an otherwise unnecessary frame.
- Missing or failed drawable submission retains the request without a busy loop.
- Successful submission consumes the request and restores prior AppKit mode.
- Nested AppKit transaction presentation is preserved.
- Non-macOS defaults and ordinary asynchronous rendering remain unchanged.

Run the affected linked GPUI/Apple/macOS tests, formatting and upstream lint
checks. Build and manually run Zed from that exact final revision: editor input,
save, palette dismissal/reopening, dock toggles, resize and window activation.
Report unavailable Windows/Linux coverage rather than claiming native checks.
Do not port Con's private snapshot packaging into the upstream source PR.

## Submission Gates

- Confirm maintainer interest in the discussion. Offer a small standalone GPUI
  native-view example if the Con capture is insufficient for their review.
- Before editing upstream source, follow its README review-marker rule. Only
  the human author may remove the marker after personally reviewing the work.
- Have the human author approve the final description and self-review checklist.
  Do not check the human self-review boxes on their behalf.
- Disclose AI assistance. If translating the author's Chinese explanation,
  quote the translation and retain the original as requested by upstream.
- Attach the matched embedding clips and describe the separate Zed smoke scope.
  Do not claim a Zed-editor reproduction or 120 FPS without evidence.
- Open a focused PR with an imperative title such as "Allow frame-scoped native
  view presentation on macOS", its discussion link and a `[GPUI]` release note.

The existing compatible fork remains pinned until official packages provide
equivalent behavior. See [the integration and replacement record](native-terminal-presentation.md)
for the immutable pin, alpha distinction and removal criteria.
