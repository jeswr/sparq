---
name: ast-grep
description: Understand code STRUCTURE without reading whole files — outline a large file to its signatures, structural-grep every impl of a trait, and find call sites — using ast-grep (tree-sitter structural matching) and the editor outline/LSP, and knowing WHEN to reach for ast-grep vs Grep vs LSP vs a full Read. Use on the 37-crate / ~568-file Rust workspace when a question is about code SHAPE (impls / call patterns / signatures / codemods) rather than an exact string. MANDATORY: install + verify ast-grep first, and never invoke it as `sg` (the `sg` name collides with `newgrp` on this box).
---

# ast-grep + outline: read STRUCTURE, not whole files

[OPUS-4.8] Internal agent skill. Authored by Opus 4.8 (Fable unavailable — flag
for re-review when Fable returns). Design basis:
`research/agent-effectiveness-program.md` §2.2 (the query-type → tool map) and
the shared A/B protocol in `research/dogfooding-sparq-knowledge-graph.md` §5.

The single load-bearing idea: a "where / how is X done" question over a big file
or a 37-crate workspace usually does **not** need the file's bytes — it needs the
file's **skeleton** (signatures) or a **structural** match (all impls of a trait,
all call sites of a function). Reading the whole file to answer it spends tokens
proportional to the *file*, not the *answer*. ast-grep gives you the answer-sized
view: tree-sitter structural matching with `file:line` hits, zero runtime deps.

> **This is a tool, not a verdict.** Whether ast-grep + outline actually beats the
> `Grep` + `Read` baseline on agent tokens for this repo is decided by the §5 A/B
> (see the bottom of this file), not by this skill. Use the recipes; do **not**
> cite an unmeasured saving.
>
> **Honest scope (a firm real-token A/B has now settled this — §5).** ast-grep +
> outline are **precision / completeness tools, measured NOT to save tokens** — the
> firm A/B (`bench/pkg-dogfood/RESULTS-astgrep.md`) found going outline/ast-grep-FIRST
> on a code-structure question is **slightly MORE expensive end-to-end** than a scoped
> `Read`. ast-grep's edge over a *competent* `grep -rn` is **precision and
> completeness**: it skips the same token inside comments/strings and matches *shapes*
> a line-regex cannot (`$X.map($$$).collect()`, a call with a specific arg), and in the
> A/B that bought a small **quality** nudge on call-site completeness — **not** a raw
> byte/token cut. The **one** lever that survives the firm A/B is outlining **only**
> the skeleton of a *very large single file* to avoid reading it whole (§2.1, §4) — a
> genuine per-file reduction. Reach for ast-grep when grep's false positives or its
> inability to express a structure actually bite, or when you must NOT miss an
> impl/call-site; do **not** reach for it to cut tokens.

## 0. Install + verify FIRST (and the `sg` collision — non-negotiable)

ast-grep may already be installed (e.g. under `~/.cargo/bin`) but **not on
`PATH`** — so check first, then install only if missing, and **always verify**
before any rule runs:

```bash
# 1. Is it already here? (it may be installed but off PATH)
command -v ast-grep || ls ~/.cargo/bin/ast-grep 2>/dev/null
#   if it's at ~/.cargo/bin/ast-grep:  export PATH="$PATH:$HOME/.cargo/bin"

# 2. Install only if missing. Prefer the Rust CLI (pinned, reproducible, no per-turn tax):
cargo install ast-grep --locked          # builds in ~1-2 min; needs cargo on PATH
#   no cargo on PATH? use a prebuilt-binary route instead:
#     npm  i -g @ast-grep/cli            # ships a prebuilt binary as `ast-grep`
#     pip  install ast-grep-cli          # ditto via PyPI

# 3. VERIFY before using — every recipe below assumes this prints a version:
ast-grep --version          # -> e.g. "ast-grep 0.43.0"
```

**Invoke it as `ast-grep` — NEVER as `sg`.** ast-grep ships an `sg` convenience
alias, but on this box that name is a **collision**:

```bash
ls -l /usr/bin/sg           # -> /usr/bin/sg -> newgrp  (a DIFFERENT program)
```

`/usr/bin/sg` is a symlink to `newgrp` (switch-group). Depending on `PATH`
ordering, typing `sg` may run `newgrp` instead of ast-grep and hang or change
your group context — a silent footgun. **Every example in this skill uses the
full `ast-grep` command, and so must you.** Do not alias it back to `sg`.

If you reach for the `ast-grep-mcp` server instead of the CLI, it **must** load
behind Tool-Search deferral (per `research/agent-effectiveness-program.md` §1.6
prefix-tax reject) — an always-loaded MCP tool re-bills its definition every turn
across every parallel agent and can erase any saving. The CLI has no such tax;
prefer it.

## 1. WHEN to use ast-grep vs Grep vs LSP vs a full Read

Pick the tool by the **shape of the question**, not by habit. This is the
query-type → tool map (`research/agent-effectiveness-program.md` §2.2):

| You want… | Use | Why |
|---|---|---|
| an exact string / error text / a log line / a TODO marker | **Grep** | zero false-negatives, no setup, fails *loudly*; ast-grep would over-think it |
| every **impl of a trait** / every **call site** of a fn / a **code shape** / a codemod | **ast-grep** | matches the parse tree, so it skips the same token inside comments/strings and is robust to whitespace/wrapping that defeats line-regex |
| a file's **skeleton** (its fns/types/signatures) before deciding what to read | **outline** (editor outline / ast-grep signature recipe §2.1) | answer-sized view of a 1000-line file without its bytes |
| **who calls / where defined / type-of / rename-safely** (resolved symbol graph) | **LSP** (`find_usages`, go-to-def, rename) | resolved edges — zero false positives, follows re-exports & generics that ast-grep's *syntactic* match cannot resolve |
| the actual **logic / control flow / a specific section** you've already located | **Read** (offset+limit on the located lines) | once outline/ast-grep/LSP gave you `file:line`, read only that span |

Decision flow for "understand this code":

1. **Big file, unsure where the relevant part is?** → outline it (§2.1), then
   `Read` only the located span. Do not `Read` the whole file first.
2. **"All the X's" across the workspace** (impls, call sites, a pattern)? →
   ast-grep (§2.2–§2.4). One command beats opening N files.
3. **Need resolution** ("who *actually* calls this, through the re-export",
   "every override of this trait method by type")? → LSP. ast-grep is
   *syntactic* — it matches text-as-parsed, not resolved symbols (see the
   path-qualification gotcha in §2.3).
4. **Exact literal**? → Grep. Don't reach for a parser to find `"unsupported
   datatype"`.

The honest boundary: if a file is **short** (≲200 lines) and you need most of it,
just `Read` it — the outline round-trip is pure overhead. ast-grep wins on
*large* files and *workspace-wide "all the X"* questions, exactly where a full
read is most wasteful.

## 2. Recipes (every one verified on THIS repo)

All commands below were run against `crates/` on this workspace and returned the
stated kind of result. ast-grep prints `file:line` headings by default; pipe
`--json=compact` to `jq` when you want a one-line-per-hit digest.

> **Pattern mental model:** `$NAME` = one named node (a metavariable); `$$$` =
> zero-or-more nodes (e.g. an arg list or a body); a bare token matches itself.
> A pattern matches the syntax **as written** — see the §2.3 gotcha.

### 2.1 Outline a large file → signatures only (don't read the bytes)

Get every function's signature + line number, so you can `Read` only the one you
need:

```bash
# All fn signatures in a file, as "line: signature":
ast-grep --lang rust --pattern 'fn $NAME($$$ARGS) $$$' crates/sparq-core/src/dictspill.rs \
  --json=compact | jq -r '.[] | "\(.range.start.line + 1): \(.lines | split("\n")[0])"'

# Only the PUBLIC surface of the file:
ast-grep --lang rust --pattern 'pub fn $NAME($$$) $$$' crates/sparq-core/src/dictspill.rs \
  --json=compact | jq -r '.[] | "\(.range.start.line + 1): \(.lines | split("\n")[0])"'

# The type skeleton (structs) of the file:
ast-grep --lang rust --pattern 'struct $NAME { $$$ }' crates/sparq-core/src/dictspill.rs \
  --json=compact | jq -r '.[] | "\(.range.start.line + 1): \(.lines | split("\n")[0])"'
```

`jq` takes the first line of each match, so you get the signature line, not the
body. (An editor/LSP **document outline** does the same thing with one call and
also nests methods under their `impl` — prefer it when you have it; the ast-grep
recipe is the dependency-free fallback that also works workspace-wide.)

### 2.2 Find every call site of a function

```bash
# Every "<receiver>.to_spo(...)" call across the workspace, one line each:
ast-grep --lang rust --pattern '$RECV.to_spo($$$)' crates/ \
  --json=compact | jq -r '.[] | "\(.file):\(.range.start.line + 1): \(.lines | split("\n")[0])"'

# A free function's call sites (no receiver):
ast-grep --lang rust --pattern 'load_reader_parallel($$$)' crates/ --json=compact | jq length
```

This finds calls **as parsed**, so it ignores the same token inside a doc comment
or a string literal — the classic `Grep` false-positive. For *resolved* callers
(through re-exports / trait dispatch) prefer **LSP `find_usages`**; ast-grep is
the fast, zero-setup first pass.

### 2.3 Find every impl of a trait — the path-qualification gotcha

The naive pattern under-matches. On this repo, `impl std::fmt::Display for $T`
and `impl fmt::Display for $T` are written **both** ways, and a bare
`impl Display for $T` pattern matches **neither** — because ast-grep matches the
syntax *as written*, and `std::fmt::Display` is a single scoped-path node, not a
bare `Display`. So:

```bash
# WRONG — under-matches; returns 0 here because nobody writes the bare form:
ast-grep --lang rust --pattern 'impl Display for $T { $$$ }' crates/   # -> 0 hits (misleading!)
```

The robust, path-agnostic way is a **YAML rule** that matches the `impl` node and
regex-matches the trait's final segment (works for `Display`, `fmt::Display`,
`std::fmt::Display` alike). This skill ships it as
[`rules/impl-of-trait.yml`](rules/impl-of-trait.yml):

```bash
# All impls of Display, regardless of how the path is qualified:
ast-grep scan --rule .claude/skills/ast-grep/rules/impl-of-trait.yml crates/ \
  --json=compact | jq -r '.[] | "\(.file):\(.range.start.line + 1): \(.lines | split("\n")[0])"'
```

To target a different trait, edit the rule's `regex:` line (e.g.
`'(^|::)Iterator$'`). The rule body:

```yaml
id: impl-of-trait
language: rust
rule:
  kind: impl_item            # an `impl ... { }` block
  has:
    field: trait             # the trait being implemented
    regex: '(^|::)Display$'  # final path segment == Display (any qualification)
```

**Lesson, generalised:** for anything the source qualifies inconsistently
(`std::fmt::X` vs `fmt::X`, `Result` vs `io::Result`), a `--pattern` is brittle —
use a `kind: ... + has/regex` YAML rule, or fall back to **LSP**, which resolves
the symbol and is qualification-proof by construction.

### 2.4 Find a call to fn Y with a specific argument shape

ast-grep's edge over Grep is matching *structure*, e.g. a call whose argument is
itself a particular expression:

```bash
# every `.collect()` whose receiver is a `.map(...)` chain (a shape, not a string):
ast-grep --lang rust --pattern '$X.map($$$).collect()' crates/ --json=compact | jq length

# a macro call with a specific first arg:
ast-grep --lang rust --pattern 'assert_eq!($A, $B)' crates/sparq-core/ --json=compact | jq length
```

### 2.5 Preview a codemod (dry-run, no write)

ast-grep can rewrite the matched node — but **preview first** (no `--update-all`
flag = dry-run diff, nothing written):

```bash
# show the diff a rename WOULD make — does NOT touch disk without --update-all:
ast-grep --lang rust --pattern '$R.to_spo($X)' --rewrite '$R.spo_of($X)' crates/sparq-geo/src/index.rs
```

Only add `--update-all` once the preview is exactly right. A structural rewrite
is safer than `sed` because it edits parse nodes, not lines — but still review
the diff and re-run the build/clippy gate (`AGENTS.md`) afterwards.

## 3. Quick troubleshooting

- **0 hits but you expected some?** Your `--pattern` is probably matching a
  qualified path literally (the §2.3 trap). Loosen with a metavariable, switch to
  a YAML `kind` rule, or use LSP.
- **Too many hits?** Add structure — `$RECV.foo($$$)` (only method calls) instead
  of `foo` (the bare identifier anywhere).
- **`ast-grep: command not found`** → you skipped §0; run the install + verify.
- **A pattern won't parse?** Inspect the tree to learn the node `kind`s:
  `ast-grep --lang rust --debug-query --pattern '<your pattern>' a_file.rs`.

## 4. Outline-before-read discipline (applies with or without ast-grep)

Independent of any tool, and the **one lever the firm §5 A/B leaves standing** —
outlining **only** the skeleton of a *very large single file* to avoid reading it
whole is a genuine per-file reduction (a 1683-line file's signatures are a small
fraction of its body). Note the firm A/B's scope: this is about *one large file*; the
broader "outline/ast-grep-FIRST as a standing strategy" was measured **more** expensive
end-to-end (§5), so do not over-extend this into "outlining always saves tokens".

> **For one *large* file (> ~200 lines) whose relevant span you must locate, get an
> outline/skeleton (functions + signatures) FIRST and read only the section(s) you
> need — do not `Read` the whole file to answer a "where/what is X" question.**

When to outline vs read whole:

- **Outline first** when you need to *locate* something, *understand the shape*
  (what does this module expose? where is the fn that does X?), or decide *what to
  read* — i.e. the question is about structure, not a specific body.
- **Read whole** when the file is short (≲200 lines and you need most of it), or
  once the outline/ast-grep/LSP has given you the exact `file:line` span — then
  `Read` with `offset`+`limit` on just that span.

The outline can come from the editor/LSP **document outline** (preferred — it
nests methods under their `impl` and resolves symbols) or from the §2.1 ast-grep
signature recipe (the dependency-free, workspace-wide fallback).

> **Scope note.** This skill is the *home* of the outline + query→tool discipline,
> because `.claude/skills/` is read by agents. Baking the same discipline directly
> into the role briefs (`.claude/agents/*.md`) is a **separate, out-of-scope**
> change tracked as its own bead — that surface is protected (auto-mode blocks
> agent-config self-modification), so do **not** edit it from here.

## 5. Is this actually worth it? — the firm A/B verdict

**The firm real-token A/B has now RUN (sq-0fb3f), and the verdict is: NOT a
token-saver.** The sanctioned record is
[`bench/pkg-dogfood/RESULTS-astgrep.md`](../../../bench/pkg-dogfood/RESULTS-astgrep.md)
(numbers live there; non-canonical work-box telemetry, never frozen into this file).
N=16 code-structure questions, Opus both arms, real cache-discounted effective input
tokens (`1.0·fresh + 0.1·cache_read + 1.25·cache_creation`) with a paired quality grade:

- **Arm A = normal full `Read`** was **cheaper** at the median and on **11 / 16** tasks.
- **Arm B = outline / ast-grep-FIRST** cost **more** at the median (~21k more effective
  input tokens) and was cheaper on only **5 / 16** tasks — **more expensive across
  every question kind** (call-sites, structure, signature, trait-impls).
- B's only edge was a **small quality nudge** on call-site **completeness** (A→B on
  that one kind). Completeness, not cost.

**Conclusion:** outline/ast-grep-FIRST is **not** a token-saver end-to-end — the
install + structural queries + verification reads cost more than a scoped `Read`. Use
ast-grep + outline as **precision / completeness** tools (enumerate ALL impls/call-sites
where a `Read`/`grep` might miss one; express a shape a line-regex cannot), not to cut
tokens. The **narrow exception** that survives: outlining ONLY the skeleton of a *very
large single file* beats reading it whole (§4).

**History — the 2nd proxy reversal.** Before the firm A/B, a directional **byte
proxy** (sq-lhwo.2) — a model-free file-read byte comparison over a few "where/how is
X" tasks — produced an "outline is the big lever" signal. That proxy **overstated** it:
counting outline-skeleton bytes vs whole-file bytes ignores the install cost, the
multiple structural queries, the re-reads, and the verification reads the real strategy
pays, and it had no quality pair. The firm real-token A/B corrected the verdict — the
second time a charitable byte/char proxy inverted a verdict that real-transcript
measurement then fixed (the first: the #1078 PKG-query char-proxy; see
[`RESULTS.md`](../../../bench/pkg-dogfood/RESULTS.md)). **Takeaway:** a byte proxy is a
direction-finder, not a substitute for mining the tokens the model actually consumed.
The "20–40% file-read reduction" floated in
`research/agent-effectiveness-program.md` §2.2 is a third-party advisory estimate with
**no weight** in the decision.
