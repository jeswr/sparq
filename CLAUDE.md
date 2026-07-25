# CLAUDE.md

This repo's agent instructions live in **[`AGENTS.md`](./AGENTS.md)** — read it first.

It is the single source of truth for: what sparq is, how to USE it (the `skills/` tree),
how to work ON it (build/test/lint gate + merge discipline), the public-API → `SKILL.md`
maintenance rule, and **repository hygiene** — where every kind of output belongs:

- **Tasks / TODOs / future work → a GitHub issue** (`gh issue create`), never a markdown `TODO`/`TODO.md`/checklist. (Beads/`bd` was retired 2026-07-17 — see `docs/bd-migration.md`.)
- **Durable knowledge → `AGENTS.md` / `CLAUDE.md` / a `skills/<surface>/SKILL.md` / a crate
  `README.md` / a `research/` design record** — whichever fits.
- **No narrative scratch docs** (`HANDOVER*.md`, `SESSION*.md`, progress logs) in the repo,
  and **no hard-coded performance numbers** in markdown.

(This file is intentionally thin so there is one source of truth. Put new content in
`AGENTS.md`, not here.)
