# Accepted Notarization Followed by a Failed Ticket Download

## Incident

The Intel macOS jobs for beta.129 and beta.130 failed after building and signing
their app bundles. Both notarization submissions were accepted. `stapler`
then lost its connection to Apple's ticket delivery endpoint and returned 68.
The macOS workflow stopped before publishing its artifacts, and the complete
release gate correctly kept both releases in draft.

Evidence:

- [beta.129 run](https://github.com/nowledge-co/con-terminal/actions/runs/38340444059):
  submission `d908a9fe-c27b-4c59-9461-aea4a0d0b747` accepted, followed by
  `NSURLErrorDomain -1005`, "The network connection was lost."
- [beta.130 run](https://github.com/nowledge-co/con-terminal/actions/runs/38390475334):
  submission `1a0ac287-a149-4245-af15-b003199fbc44` accepted, followed by the
  same error at `api.apple-cloudkit.com`.

These logs establish a ticket-download connection failure, not a compile error,
invalid signature or rejected notarization. They do not establish whether the
connection failed at Apple, GitHub's runner network, or an intermediate service.

## Fix

Add a bounded wrapper for both ticket stapling and validation. Retry only exit
68 (`EX_NOHOST`, which includes unexpected ticket-service responses), four
attempts in total, with 10/20/40-second backoff. Keep all command arguments and
the final failure status. Do not resubmit the notarization or rebuild/sign the
artifact merely to retry downloading a ticket.

Invalid, revoked, malformed and unwritable tickets stop immediately. A repeated
68 still fails the release. No signature, notarization or publication gate is
relaxed. The helper is used only by the macOS packaging scripts; Windows/Linux
packaging and runtime behavior do not change.

## Verification

Shell fixtures exercise immediate success, eventual staple/validation success,
retry exhaustion, permanent failures, and a permanent failure following a
transient one. They assert attempt counts, backoff, preserved flags and paths
containing spaces. Run them in portable CI and before native macOS release
builds. Apple service recovery still requires a real signed release run; mocks
cannot guarantee its availability.

## Lessons

Distinguish build, signing, notarization acceptance, ticket delivery and release
publication. Retrying a network operation must remain bounded and fail closed.
Do not mark an incomplete draft public, move its immutable tag, or infer a
global Apple outage from one runner's connection error.
