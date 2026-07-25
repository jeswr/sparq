# Fable context-broker (v1) — keeping Claude Fable usable across the codebase [OPUS-4.8]

> 🤖 SPARQ agent — operating protocol + tooling record for bead `sq-lhwo.5`
> (child of the agent-efficiency epic `sq-lhwo`). This is **advisory** tooling; it
> is deliberately **not** wired into the CI gate.

Status: **v1 implemented** (`scripts/fable/`). The authoritative signal is the
observed serving tier from real runs; everything else here is an advisory prior.

> **Update (2026-07-24):** Opus 5 (`claude-opus-5`) is now the PRIMARY top tier, replacing
> both the Fable 5 and Opus 4.8 heads (maintainer directive). The `fable → opus` pair this
> record characterises is the ORIGINAL instance of the primary→downgrade pattern; the
> mechanism is unchanged, and `scripts/fable/detect-tier.sh` now tracks `claude-opus-5` as
> the primary with `claude-fable-5` / `claude-opus-4-8` as the downgrade tiers. The text
> below is accurate history of the original pair — read its tier names accordingly.

## 0. The problem

Claude Fable 5 carries extra dual-use safety measures. Empirically, a Fable run
**silently downgrades to Opus** (`claude-fable-5` → `claude-opus-4-8`) mid-run, and
stays there, once certain content enters its context. The agent is **not told**, and
its self-report of its own model is unreliable — so a run can *look* like Fable while
actually being served by Opus. Naively stamping such a run `[FABLE]` is a correctness
error (cf. PR #1321).

The broker's job is to (a) let us **optimistically try Fable** for high-value
reasoning, (b) **detect the tier that actually ran**, and (c) **avoid re-paying** the
classification cost on unchanged files.

## 1. The empirical trigger model (treat as the design spec, not as proven law)

From direct probes (each a background `model:'fable'` sub-agent dispatched from an
Opus main thread; served tier read back by grepping the sub-agent transcript for
`claude-fable-5` vs `claude-opus-4-8`):

- **Benign natural-language / docs do NOT trigger the downgrade**, regardless of
  length. A ~150-line README stayed fully on Fable; a trivial crate-list + short
  README + arithmetic stayed on Fable.
- **Reading SOURCE CODE of any language triggers it** — even a pure-arithmetic,
  security-term-free file (e.g. `numeric.rs`), and even a short code slice. The
  trigger here is *"this is code"*, not *"this is dangerous"*.
- **SECURITY / crypto / adversarial TOPIC content triggers it**, even as prose (e.g. a
  short MPC/ZK design doc using threat-model / adversary / malicious-security
  vocabulary), and even when the reading is purely *defensive*.

So the reliable lever is simple: **give Fable BENIGN PROSE — never raw code and never
security-topic content.** An earlier "it's just volume" hypothesis is falsified (short
code slices trip it; long benign prose does not); so is a "security-vocabulary-only"
hypothesis (`numeric.rs` has none and still trips).

> **This model is an advisory PRIOR, not an oracle.** It predicts *code* and
> *security* files will downgrade, but it can be wrong in both directions. The only
> authoritative signal is the **observed** downgrade from a real run. When prediction
> and observation disagree, observation wins.

## 2. The protocol: optimistic-first → detect → stamp → record

1. **Optimistic-first dispatch.** Default high-value *reasoning* work (design,
   decomposition, review-of-a-digest, spec/paper **prose** authoring, synthesis) to
   `model:'fable'`. Graceful downgrade means a failed bet costs only ~1–2
   Fable-priced early turns before Opus takes over — Opus is the correct fallback
   anyway, so the run still completes correctly.

2. **Detect the actual serving tier** post-hoc. Run
   `scripts/fable/detect-tier.sh <transcript.jsonl>` over the agent's transcript. It
   prints the per-tier tally, the **dominant** serving tier, and whether/where a
   `fable → opus` downgrade occurred.

3. **Stamp the tier that REALLY ran** — honestly. If `detect-tier` reports a
   downgrade or a majority-Opus run, stamp the artifact with its **actual** tier
   (`claude-opus-4-8`), **never** `[FABLE]`. Only a run that stayed on Fable
   throughout may be stamped `[FABLE]`.

4. **Record the observation into the cache.** Feed the observed tier back with
   `scripts/fable/classify-cache.py set <file> --observed-tier {fable|downgraded}
   --classification {benign_prose|code|security}`. The next dispatch that would read
   that file can consult the cache instead of re-probing. Because the entry is keyed
   by content sha256, it is invalidated automatically when the file changes
   (`stale-check` returns `true`), so a file is only re-classified after it is edited.

### Cache entry schema

Stored in `.claude/fable/file-classifications.json`, keyed by path:

| field            | values                                             | meaning |
|------------------|----------------------------------------------------|---------|
| `content_sha256` | hex                                                | invalidation key — the file's content hash |
| `observed_tier`  | `fable` \| `downgraded` \| `null`                  | tier a real run was served after reading this file (`null` = never observed) |
| `classification` | `benign_prose` \| `code` \| `security` \| `unknown`| the advisory prior |
| `source`         | `observed` \| `predicted`                          | whether the row came from a real run or the a-priori classifier |
| `updated`        | ISO-8601 UTC                                        | last write |

**`source=observed` beats `source=predicted`.** A predicted `benign_prose` that a
real run shows as `downgraded` must be overwritten with the observation.

## 3. The prose-digester convention (how Fable reaches code/security files)

Fable cannot read raw code or security content without downgrading, and a *redacted*
copy of code still reads as code. The bridge is therefore **not** to sanitise the file
in place, but to **replace it with benign prose**:

> To have Fable reason about a code or security file, dispatch a **Haiku/Sonnet**
> sub-agent (per `scripts/fable/digester-brief.md`) to read the file and emit a
> **benign, plain-English digest** — no code snippets, no attack/exploit framing,
> sized to the Fable task's goal — and hand **that digest** (not the file) to Fable.

The cheap model does the reading; Fable reasons over the clean prose. This lets Fable
engage the whole codebase indirectly (design review, logic review, spec authoring)
while staying on Fable.

### Honest limitation

A prose digest lets Fable review **design and logic**. It **cannot** substitute for a
genuine **security review** of the redacted substance: the digest deliberately strips
the exact adversarial/implementation detail a real security review must examine, so
any conclusion Fable draws from a digest of a security-sensitive file is about the
*described design*, not the *actual code*. Route genuine security review, verifier-code
reading, and adversarial soundness work to **Opus** (which is also where a downgraded
Fable run lands anyway).

## 4. Routing summary

- **Fable (keep on Fable):** design / decomposition / synthesis, review of a
  digest, spec + paper **prose** authoring from a pre-extracted digest, UX / KM /
  measurement-design reasoning — all driven by *distilled prose inputs*.
- **Opus (route here by design; don't fight the safeguard):** reading code or
  security files, code review, implementation, and all genuine adversarial / soundness
  / verifier-code security review — anything touching `sparq-zk` / `sparq-mpc` /
  `sparq-trust`.

## 5. Tooling (this bead)

| file | purpose | invoke |
|------|---------|--------|
| `scripts/fable/detect-tier.sh` | post-hoc serving-tier observer | `scripts/fable/detect-tier.sh <transcript.jsonl>` |
| `scripts/fable/classify-cache.py` | hash-keyed classification cache | `classify-cache.py {get\|set\|stale-check\|list} …` |
| `scripts/fable/test_classify_cache.py` | stdlib unit tests | `python3 scripts/fable/test_classify_cache.py` |
| `scripts/fable/digester-brief.md` | prose-digester agent-brief template | paste into a Haiku/Sonnet sub-agent |

None of these are on the CI gate — the broker is an operating aid for the orchestrator,
not a build requirement.
