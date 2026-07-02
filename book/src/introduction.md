<!-- [OPUS-4.8] sq-im8u — single-source include wrapper. The lead paragraph is
{{#include}}d verbatim from the canonical README.md `lead` anchor (build-time content
injection), so it cannot drift from the README. The only non-included content below is
the load-bearing experimental-status caveat (a required honesty caveat — see AGENTS.md)
and the one-line "next" navigation, whose link targets are mount-point-specific (absolute
GitHub URLs that resolve under the Pages mount, unlike the README's repo-relative links).
No prose is duplicated from the README. -->

# Introduction

{{#include ../../README.md:lead}}

> **Status: experimental research engine.** The API is unstable and pre-1.0. Conformance against
> the W3C SPARQL, SHACL, and inference suites is tracked by CI ratchets that only ever go up.
> SPARQL `SERVICE` federation ships behind the opt-in `service` cargo feature (off in the
> default build); when built in it is default-DENY-all egress, allowlisted per host as an SSRF
> guard (see
> [`research/roadmap.md`](https://github.com/jeswr/sparq/blob/main/research/roadmap.md)).

## Where to go next

- [Install & build from source](./getting-started/install.md) — get a working build.
- [Capabilities at a glance](./getting-started/capabilities.md) — what each opt-in surface does.

Per-surface how-to guides live in the
[usage skills](https://github.com/jeswr/sparq/blob/main/skills/SKILL.md) router, and the full crate
map is in [`AGENTS.md`](https://github.com/jeswr/sparq/blob/main/AGENTS.md). Live per-commit
performance metrics are on the
[benchmarks dashboard](https://sparq.jeswr.org/dev/bench) — numbers are deliberately **not**
baked into these docs, because they drift.
