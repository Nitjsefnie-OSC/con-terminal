# Agent Event Draining Must Yield to Window Input

## Report and Evidence

The maintainer reported that the window sometimes could not be dragged while
the built-in agent was responding. Background Bazel compilation was also active.
That environmental load has not been ruled out for the exact reported gesture.

Source inspection found an independent input-starvation risk: the foreground
workspace task drained each agent queue until empty and only awaited a timer
when no event or terminal work occurred. An always-ready producer could extend
one update indefinitely or keep the surrounding task running without yielding
to AppKit/GPUI input dispatch. This establishes an unsafe scheduling path, not
proof that it was the sole cause of the reported gesture.

## Change

Process at most 64 queued agent events per tab per turn, keeping FIFO order and
leaving the rest queued. Always await between turns: four milliseconds while
busy, eight while idle. This preserves the existing idle cadence and bounds busy
poll frequency without dropping tokens, approvals, tool events or completions.
No provider, harness protocol, terminal presentation fork or opacity changes
are involved.

Keep the existing retained tab activity entity for animation. Three small bars
are painted in its existing fixed slot for indeterminate expanded-tab activity;
determinate progress remains a ring. Reduced Motion and clipping must still
stop animation requests. Do not make the chat transcript render on each spinner
frame.

## Verification Scope

Queue tests exercise an always-ready producer and multiple turns over a FIFO
backlog. Activity tests cover periodic staggered bars, reduced motion, clipping,
visibility and stationary progress. Existing agent-panel tests cover approvals,
streaming completion and narrow/scaled controls.

Local macOS checks passed: eight activity tests, 25 agent-panel tests and 179
workspace tests (including the two queue-fairness cases), formatting, Clippy
with warnings denied, and the linked app/CLI build. An isolated native preview
covered light-theme metadata and expanded traces, then a roughly 290-point
dark-theme sidebar with its composer, code block and expanded trace. The tab
bars were observed during an indeterminate terminal-status fixture, not a live
LLM request. The fixture command completed visibly, but the CLI completion
probe timed out; that probe is not recorded as a passing integration test.

The original live-model drag failure needs manual reproduction or a main-thread
sample before claiming it is fully resolved. Isolated fixture UI checks are not
a measurement of the user's Bazel load or a live-provider performance benchmark.
Windows/Linux native GUI validation is unavailable locally; no cross-compilation
is performed.
