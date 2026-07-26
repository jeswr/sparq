# banned-terminology fixtures

> 🤖 SPARQ agent

Deliberate-violation + must-pass fixtures for `scripts/check-terminology.py`
(issue #3811). They are `exemptPaths`-listed in `scripts/banned-terminology.json`
so they never red the repo-wide sweep, and `scripts/tests/test_banned_terminology.py`
scans them EXPLICITLY (explicit mode bypasses path exemptions) so the gate has to
actually catch them.

`violation.*` reproduce the exact shapes that reached PR #3451 with a green gate —
a `pub` Rust type and a published `.ttl` `rdfs:comment`. Neither extension was in
the old markdown-only surface.
