# Workspace canvas preview: layout ownership

## What happened

The local macOS canvas preview gave the terminal a rounded frame but left the
composer below it in a separate chrome strip. User testing exposed an incoherent
boundary, unnecessary sidebar rules, missing sidebar controls in the caption,
and a terminal visible behind the returning horizontal caption row. The light
theme made excessive gutters and material differences particularly visible.
This is an unreleased prototype, not a production incident.

## Confirmed causes

- The canvas wrapper owned only the pane tree, not the terminal and input region.
- The old sidebar divider rules remained alongside the new canvas gutters.
- Caption docking removed the left-sidebar visibility control.
- Caption docking also used the saved pinned preference while a new-tab reveal
  temporarily forced a 44pt rail. The toolbar was therefore assigned to a
  sidebar that could not contain it. Interpolating its width across the terminal
  did not provide a valid containing rectangle during the transition.
- Changing to horizontal tabs retained the outgoing caption spring, even though
  horizontal tabs immediately needed the complete global row.
- A GPUI-colored safe-area extension showed a material step against the native
  terminal despite equal RGB and opacity inputs. The underlying color/compositing
  mechanism has not been established; equal parameters are not visual proof.
- An undecorated single terminal leaf returned before the new pane-mask
  prepaint callback. After a split was reduced to one pane, that surviving
  surface could keep its old half-sized mask even with a full-sized native
  frame. This explains the reported half-window cutout, unlike earlier cache
  hypotheses alone.
- Drop-preview bounds divided the full container without reserving the 1pt
  dividers rendered by flex layout. The preview also kept old rectangular
  fills and resize feedback despite rounded pane cards.

## Changes

The canvas now owns the terminal and its bottom safe area. The composer is a
centered overlay within that safe area, capped at 800pt, with terminal rows
reserved so output cannot hide underneath. Its popovers share its horizontal
geometry. A cached passive native material view supplies only the safe-area
background; it neither receives input nor covers terminal output. Native
presentation synchronization also covers changes to this view.

The macOS canvas gutter is 2pt with 8pt corners. Sidebar rules are removed,
the visibility control remains available, and horizontal mode immediately
restores the caption row. Windows/Linux retain their previous layout paths.
Caption eligibility now follows the effective rail/panel state and actual
available width. An independent caption spring has been removed: caption
progress is derived from the sidebar's width and visibility, so it cannot
continue a second animation after navigation has settled. Compact controls share a tighter spacing budget. The composer
is fixed in place and revealed through a clip; output reservation changes once
at reveal/fully-covered boundaries instead of resizing the PTY on every frame.

That initial reveal contract still prepared an empty native strip before the
input island appeared. The revised contract reveals the complete island over
the existing terminal first, then reserves rows at the shown endpoint; hiding
releases them while the island is covered. The clip excludes empty outer
padding. Output is briefly behind the overlay during the transition, not at rest.
A shallow clip starts with the text baseline visible, while one opacity value
reveals the complete group. The input subtree is uncached only during this
opacity transition because the pinned view-cache key does not include inherited
opacity. Its stable endpoint remains cached.

The follow-up visual pass extends this geometry to split panes and the agent
panel. Each terminal host gets a pane-local rounded mask intersected with the
workspace mask; the shared window transition underlay is deliberately excluded.
Unchanged masks retain their paths, and unsplit panes remove the local mask.
Compact-tab details anchor to measured tiles rather than following the pointer.
Surface labels use UI typography and no longer sit inside an extra nested track.

Further inspection found two local invalidation gaps: host/surface autoresizing
was still enabled despite GPUI owning their frames, and the mask cache trusted
equal bounds without checking its current layer attachment. Disable that second
geometry owner, verify the actual host frame before skipping sync, and rebind
cached paths when a layer is replaced. Linked tests exercise real NSScrollView
frame changes and same-size layer replacement. These establish correctness gaps
in the prototype; live drag acceptance is still required to establish that the
reporter's complete gray-region failure is resolved.

## Verification and remaining acceptance

Focused macOS tests cover safe-area geometry, bounded width, caption clearance,
native material caching/hit testing/hiding, clipping, input, tabs and panes.
The macOS app and CLI build and strict Clippy checks passed. A real isolated
session accepted a new tab and executed a harmless printf through the normal
visible-shell CLI execution flow; its output was read back through the control API.

Preview inspection includes light/dark and expanded/compact navigation. Some
automated window captures lagged the reported UI state and terminal readback;
this limits visual acceptance, not a reason to declare presentation correct.
Rapid mode changes, the new-tab upper-left boundary, terminal scrolling and
material continuity still need user acceptance. No Windows/Linux runtime or
frame-rate benchmark was performed. This work does not close the separate
internal-display/battery native-presentation regression.

The pane follow-up passed 111 focused macOS tests, including linked native
tests for intersecting masks, sibling isolation, cache reuse and mask removal.
The app/CLI build and strict Clippy checks passed. A temporary surface reached
shell-ready state and was removed without closing its pane. However, live
inspection also showed stale GPUI chrome and gray native regions until a window
resize refreshed presentation. Sampling found the main thread in its event
loop, not blocked in the new clipping calls. The cause is not established and
this is not a passed visual acceptance. Keep the prototype local until the
presentation mismatch and light/dark appearance have been confirmed.

The next local build passed 113 focused tests, including actual host-frame
validation, same-size mask detachment/layer replacement and the single-stage
input reveal. Strict Clippy and the app/CLI build passed. In a temporary
three-pane tab, dragging the vertical divider and releasing preserved rounded
edges without gray residuals. That tab was closed and the original three tabs
were preserved. A title-drag attempt did not perform a reorder, so it is not
reported as a passed reorder test. Direct background launch also reproduced a
separate preview limitation: traces showed no new prepaint/paint after API
changes until Finder activated the already-running bundle. Distinguish that
frame-delivery state from a failed GPU submission or the reporter's drag case.
The final preview is activated, but prolonged dragging, dark-theme desktop
inspection and the internal-display/battery scenario still need acceptance.

A subsequent report showed a gray terminal region after a pane was dragged to
the temporary horizontal strip in vertical-tab mode. Code inspection confirmed
that pane drags unnecessarily exposed a second tab destination and undocked
the caption. Vertical mode now keeps its existing sidebar destination and
caption geometry. Pane reorder and promotion explicitly invalidate native
layout for reused surfaces. These close identifiable layout-ownership gaps;
they do not by themselves prove the complete reported gray-region failure is
resolved. Automated title drags on this machine have not performed a reorder,
so the next preview still needs that manual acceptance.

This follow-up passed 118 focused macOS tests, formatting, strict Clippy and an
app/CLI build. The restarted preview was activated through Finder and inspected
with three panes, the narrower command island and the compact-tab hover card.
The original two tabs were retained; the disposable test tab was removed.
Windows/Linux native layout paths and upstream dependencies are unchanged, but
their runtime rendering was not tested. Shared hover/composer styling is part
of this visual change; do not describe those styles as macOS-only.

The next regression fix removes the macOS single-pane early return so every
terminal leaf clears or replaces its local mask during prepaint. A linked GPUI
test seeds an old split clip on a real Ghostty view, renders it through the
single-pane path and checks that the full resolved bounds replace it at radius
zero. A nested-tree geometry test checks divider reservations for both axes.
Rounded drop regions and narrow resize feedback replace the old rectangular
overlays. Search controls and the expanded navigation toolbar are tightened;
small prototype shadows follow the maintainer's explicit request and leave
native terminal opacity unchanged. Runtime acceptance and performance limits
remain separate from these code-level checks.

The first elevation preview exposed another composition mistake: a normal
drop shadow painted by a transparent GPUI wrapper shaded the native surface's
interior because no opaque GPUI fill followed it. The corrected primitive
clips shadow painting to four exterior strips. A geometry test verifies that
those strips do not intersect the content at small, composer and large canvas
sizes. This avoids both the gray interior and full-content shadow fragments.

The corrected follow-up passed 128 focused macOS tests, formatting, strict
Clippy and the app/CLI build. In a disposable two-pane tab, the right pane was
closed through its visible header control; the surviving terminal filled the
canvas without retaining the half-pane mask. The disposable tab was removed
and the original three tabs retained. The final masked-shadow preview was
inspected after activation and no longer shaded the terminal interior. These
are settled-state checks, not high-speed frame capture or a frame-rate
benchmark. Portable drop-bound calculations changed to include dividers;
Windows/Linux runtime acceptance remains unavailable.

The caption follow-up also accounts for the combined rail and Files/Search
column, rather than treating an open tool column as a 44pt rail. A rail on its
own still retains the global caption. Pane headers and surface tabs now reuse
the cached terminal identity asset without process inference during rendering.
The follow-up passed all 129 focused macOS tests and strict Clippy. The first
test compilation was interrupted after a prolonged loader wait; the retry
compiled successfully and completed the tests. No Windows/Linux runtime or
frame-rate claim follows from these checks.
The app/CLI build also completed. The updated light-theme preview was inspected
with Search open: window controls fit above the tool column and terminal
content reaches the top. A disposable two-pane tab confirmed the new header
icons and the input island's settled layout; the test tab was then removed.

## Lessons

### Differently colored TUI surfaces

The nvim screenshot exposed why same-color shells are insufficient for edge
acceptance. The native clipping function returned immediately for a host without
a backing layer, although the real host is an NSScrollView and its Metal-backed
descendant does not guarantee host backing. The prior native fixture explicitly
enabled backing, so it could not catch this path. Rounded clipping now promotes
only the host when needed; a new NSScrollView fixture starts unbacked and checks
mask attachment without promoting the window parent. This closes the skipped
mask path, not every possible source of a one-pixel seam.

Application identities now reuse retained process observations and bundled
font glyphs. The observer refreshes a shared per-terminal icon only when it
changes; tab and surface rendering do not run OS queries. Names are matched
exactly (including executable basenames and Python versions), editor processes
outrank plugin runtimes, and conflicts stay generic. Terminal output is not
application-identity evidence. The shorter contact shadow follows a 1.5pt blur
and 0.5pt offset, without introducing a decorative border.

That shorter shadow was also rejected in the next user feedback. The maintainer
explicitly allowed restrained hairlines instead of strict borderlessness. The
current revision removes the shadows, outlines panes and the tray with one
physical pixel, and clips quad rasterization to perimeter strips. It does not
add a second enclosing canvas outline. Application identities now use color.

An isolated nvim test also showed a separate padding-color mismatch: Ghostty's
native default fills unused grid padding with the configured terminal background,
not the application's background. Logged host/clip/surface geometry matched, so
automatic scroll insets were not changed speculatively. The native renderer now
loads `window-padding-color = extend` as a default before user configuration.
An explicit native choice still wins; a linked config test checks all three
effective values. This uses upstream's edge extension and prompt safeguards,
not a GPUI fill over terminal output.

The hairline revision passed 103 focused app tests and all 35 linked
`con-ghostty` tests, strict app Clippy, formatting, and the app build. An isolated
light-theme preview showed colored Python and Neovim identities and Neovim's
background extending into the bottom padding and rounded corners. The original
user preview was not restarted or sent test input. Dark-theme runtime inspection,
Windows/Linux runtime acceptance, and frame-rate measurement remain outstanding.

The next feedback requested a diagonal contact shadow alongside the fine edge.
The shared edge now uses a 0.5pt offset on both axes, a 1.5pt blur and -0.5pt
spread, painted only outside content. The agent panel keeps the same edge but
stays anchored. Inspection also found the tray edge was drawn before its fill
and compact navigation tools were centered 2pt away from session identities.
The edge now follows the fill; compact controls use the rail centerline, with
a rendered-geometry regression assertion. Suspected corner leakage still needs
visual acceptance; a screenshot alone does not establish its cause.

This follow-up passed all 33 frame/sidebar tests, strict Clippy, formatting and
the app build. A separate light-theme Material preview exercised compacting the
sidebar with colored identities, the outlined tray and the contrasting Neovim
surface. The earlier preview remained running. No frame capture or portable
runtime acceptance was performed; visual approval remains with the maintainer.

The subsequent corner feedback exposed two inconsistent construction rules:
the Gaussian shadow was cut into rectangular exterior strips, excluding the
part of a rounded corner's penumbra inside its rectangular bounds, and the
agent panel used a 2pt top inset while split cards started at 5pt. Contact depth
now uses three faint, nested exterior rounded contours with lower-right lighting,
without a full-surface path or Gaussian cutoff. Native masks, pane headers,
surrounds and edges share an 8pt radius. The agent panel follows card and tray
vertical insets. This replaces the previous shadow construction rather than
increasing shadow opacity to disguise its seams.

Validation: all 73 focused app, icon, sidebar, pane and frame tests passed again
after the final drag-radius consistency edit. Strict Clippy, formatting and
the app build passed. The separate
Con Canvas Curves preview exercised light and dark Neovim backgrounds under
light chrome, with pane/agent tops and tray/agent bottoms aligned. The supplied
Tail A6 PNG was copied byte-for-byte (SHA-256 verified), and the preview ICNS was
generated from it through the packaging resize sizes. Portable runtime, a dark
app theme and high-speed motion remain unverified.

Single-pane feedback found another ownership mismatch: only split leaves had
their own rounded card clip, while the outer canvas clip included the input-tray
reservation. A single terminal's bottom therefore stayed square. Canvas depth is
now passed explicitly through pane rendering; a single decorated leaf gets the
same card geometry as split leaves, while clean mode still clears the clip. The
linked single-pane test covers both transitions and checks actual clip bounds.
The enclosing native viewport is now rectangular: it reserves layout space,
while leaves alone own the rounded contour. Painting another set of enclosing
corner cutouts over leaf cutouts would duplicate antialiasing and material fills.

Compact icon containers were centered, but font-glyph ink did not match the
transparent inset baked into the bitmap identities. Font identities now fit
their actual bundled ink bounds to an 80% optical slot and center that ink using
the shaped line's baseline, not guessed per-icon offsets. No new icon library
or installed-font scan is involved. The right panel returns to unframed base
chrome, with only its composer raised. Terminal-card shading is removed; hover
details use a distinct opaque popover material and grouped pane context. The
visual pass replaced an inset pane-context fill that resembled a button with
a faint separator, and matched the agent composer's material to the input tray.

### Input tray feedback

The centered 800pt field and native safe-area extension still read as two
separate surfaces. Reserving rows only at the reveal endpoint also caused a
second layout change after the controls appeared. The follow-up replaces that
composition with a full-column tray aligned to pane edges, an 8pt radius and
6pt bottom clearance. The tray fades at a fixed size; the viewport is reserved
at opening and retained until the fade-out completes, including reversals.
The native safe-area material is disabled rather than recolored to simulate a
continuous terminal surface. Contact shadows are stronger but remain exterior
only. This does not claim a new terminal content-padding API or flicker-free
motion on untested displays.

The final tray revision passed 129 focused macOS tests, strict Clippy, formatting
and the app build. Light-theme visual inspection caught an overly broad,
clipped contact shadow; its blur/spread returned to the narrow exterior-only
construction, with the stronger material contrast and 3pt pane inset retained.
Keyboard cycling reached all three input modes and hide/show returned the tray
to its visible endpoint. The preview retained the user's three tabs and split
panes. Dark-theme runtime inspection, high-speed motion capture and portable
runtime acceptance were not performed for this revision.

Define the canvas, safe area, caption and popover geometry together. Do not call
an inset strip a floating composer or treat a unit-test count as visual approval.
Use small cached corner paths, never a viewport-sized path with a hole. Keep
native material ownership confined to the reserved area; do not solve seams by
making the entire terminal opaque.

The final single-pane/sidebar revision passed 64 focused macOS tests (including
linked native clip bounds), strict Clippy, formatting, diff checks and the app
build. The Unified preview was rebuilt and visually inspected in the configured
light theme. Composer typing and clearing were exercised without submitting a
message. The right sidebar has one base-material owner, not two overlapping
alpha fills. Windows/Linux runtime, dark-theme runtime, high-speed presentation
and frame-rate measurements remain unverified; these results are not acceptance
of those cases. Production sessions and the user's edited source artwork were
not modified by this validation.

### Corner-to-edge steps

The next close-up exposed a one-physical-pixel step at pane corners. The AA
overlap used a translated, smaller arc with short perpendicular segments at
both endpoints. Those segments broke tangency with the straight card edges.
The overlap now tapers to the original endpoints: adjusted cubic controls
retain diagonal coverage without either perpendicular segment. Regression
checks exercise both edge joins at 1x and 2x while retaining the coverage and
small-path bounds checks. This changes only cached macOS corner geometry, not
native presentation timing, terminal opacity or pane layout.

The final revision passed 33 focused pane/frame tests, strict Clippy, formatting
and the app build. Unified was rebuilt and inspected in the configured light
theme. Some initial chrome/content appeared stale until a window resize; this
corner-geometry fix does not establish that initial-layout behavior is resolved.
The agent composer being absent while the bottom tray was visible was expected:
those input locations are mutually exclusive, not a missing-composer defect.
Portable runtime and dark-theme inspection remain outside
this validation.

### Shared gutters and composer baseline

The terminal split gutter was 7pt (two 3pt insets and a 1pt divider), but the
pane-to-tray gap was 9pt and the tray-to-window clearance was 8pt. The agent
composer also added its own bottom padding below the workspace's inset.
Use the existing 7pt split rhythm for all three gaps. Derive the tray's viewport
reservation from its height and those clearances, rather than doubling a margin
that serves a different purpose. The right composer adds no second bottom
padding and follows the terminal card baseline, including the reserved tray
height. Pane cards, command tray and agent composer share an 8pt radius and a
single fine-edge treatment; only hover popovers retain contact shading. These
changes are macOS-scoped; portable styles and rendering paths are unchanged.
Resize feedback also now occupies that same gutter: a centered short rounded
grip replaces the full-length active line. Hover and drag change only its fill;
the complete 7pt gutter remains clickable and a layout/interaction regression
test checks both axes and a click well outside the visible grip.
Horizontal tabs contributed another 2pt below their capsule, making the visible
tab-to-pane gap larger than the lateral outer inset. The canvas top margin now
subtracts that existing clearance while retaining caption height and centered
tab controls. Its gutter paint, native viewport and agent-panel top share the
same geometry; bottom clearance is independent and remains unchanged.

Verification: 37 of 38 focused macOS tests passed in the first completed run.
The remaining reveal-reversal test still expected the old 56pt reservation;
after updating its explicit expected viewport from 244pt to 247pt for the new
53pt reservation, its isolated rerun passed. Grip geometry/full-gutter clicks,
scaled composer bounds/submission and linked native clipping passed. Strict
Clippy, formatting, diff checks and the app build passed. Unified was rebuilt;
light-theme inspection confirmed the right-composer baseline and the tightened
horizontal-tab clearance. Background startup initially retained partial chrome;
a process sample showed an idle main run loop and the socket reported ready
shells. Clicking the blank titlebar activated the window and completed its
visible content. No general initial-paint fix is claimed here. Portable runtime,
dark-theme runtime and high-speed motion remain unverified.

### Visible leading clearance of the agent composer

The previous alignment measured 5pt from the sidebar's invisible column edge.
The terminal card already ended 5pt before that edge, producing a visible 10pt
leading gap against 5pt trailing and bottom gaps. Move the old sidebar-wide
horizontal padding to the header, transcript and history themselves. Their
physical content positions stay unchanged; the composer can occupy the column's
leading edge with only its 5pt trailing inset. Keep the resize hitbox fully
inside the clipped sidebar after removing its outer padding. No negative
margins, translated controls or portable style changes are needed.

Verification: all 33 agent-panel tests and two workspace spacing tests passed,
including narrow and scaled composer bounds. Formatting, strict Clippy, diff
checks and the app build passed. The updated Unified light-theme preview shows
equal 5pt visible leading, trailing and bottom clearance. Windows/Linux runtime,
dark-theme runtime and high-speed motion were not validated in this follow-up.

### Release readiness

The maintainer accepted the final Unified preview and authorized PR #488 and a
beta release. A fresh Cargo run passed all 650 app tests, including the previously
timing-sensitive file watcher tests. All 35 linked Ghostty tests also passed,
including padding-color defaults and explicit user overrides. Formatting, strict
Clippy, docs routing validation and the app build passed. Shared application
identity mapping and bundled glyph coverage now run on native Windows/Linux CI.
The macOS canvas does not replace those platforms' terminal layout paths.
This acceptance is not a frame-rate benchmark or certification of the intermittent
internal-display/battery presentation seam; retain that separate acceptance work.
