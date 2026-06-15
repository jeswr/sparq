# `.claude/skills/` — internal agent skills

This tree holds **internal** Agent Skills for agents working *on* the engine's
source (parsing perf, ZK circuits, MPC, HDT, etc.). It is distinct from the
top-level [`skills/`](../../skills/) tree, which documents how to *use* sparq's
public surfaces. See the note in [`AGENTS.md`](../../AGENTS.md) ("Skills — how to
USE sparq") — do not confuse the two.

## Vendored third-party skills

- **`logo-designer/`** — vendored from
  [neonwatty/logo-designer-skill](https://github.com/neonwatty/logo-designer-skill)
  (MIT, © Jeremy Watt; see `logo-designer/LICENSE`). Branding / SVG work:
  designs and iterates on logos as SVG, exports to PNG at standard sizes via
  `scripts/export.sh`. For SVG *generation* via an external service, the
  optional [Recraft](https://www.recraft.ai/) and
  [SVGMaker](https://svgmaker.io/) APIs (REST + MCP) can be wired in with an API
  key — neither is required to use the skill.
