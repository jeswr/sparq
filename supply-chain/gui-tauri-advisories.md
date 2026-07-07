<!-- [OPUS-4.8] sq-x3r9b — advisory-disposition record for the EXCLUDED gui/src-tauri
     workspace. Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable
     returns). NON-CANONICAL timing; no measured performance numbers baked here. -->

# gui/src-tauri — advisory disposition (Tauri 2 desktop workspace)

> 🤖 SPARQ agent. This records the disposition of the RustSec / GitHub-advisory findings
> that Dependabot raises against **`gui/src-tauri/Cargo.lock`** — the Tauri 2 desktop GUI,
> which is a **standalone `[workspace]` root EXCLUDED from the root workspace**
> (`Cargo.toml` `[workspace].exclude = [… "gui/src-tauri"]`). Because it is excluded, the
> root `cargo-deny` advisory gate (`.github/workflows/supply-chain.yml#audit`, which runs
> `cargo deny check advisories` over the root `Cargo.lock`) **does not see this workspace**,
> and there is no gui-scoped advisory gate today. These findings therefore surface **only
> via Dependabot**, not via any in-repo CI gate. This file is the honest, downstream-facing
> disposition for that Dependabot-only surface.

## Headline finding — Dependabot alert #5

| Field | Value |
|---|---|
| Advisory | **RUSTSEC-2024-0429** (= **GHSA-wrw7-89jp-8q8g**) |
| Crate | `glib 0.18.5` |
| Summary | Unsoundness in the `Iterator` / `DoubleEndedIterator` impls for `glib::VariantStrIter` |
| Severity | medium (GitHub advisory DB); RustSec classifies it `informational = "unsound"` |
| Vulnerable range | `>= 0.15.0, < 0.20.0` — first patched **0.20.0** |
| Manifest | `gui/src-tauri/Cargo.lock` |
| Reached via | `tauri 2.11.3` → `gtk 0.18.2` → `glib 0.18.5` (Linux GTK3 webview stack) |

## Why the clean fix is not available here (upstream-blocked)

The fix is `glib >= 0.20.0`. It **cannot be taken in this workspace** while Tauri 2 targets
the Linux **GTK3** webview stack. The resolver rejects it directly:

```
$ cargo update -p glib --precise 0.20.0 --manifest-path gui/src-tauri/Cargo.toml
error: failed to select a version for the requirement `glib = "^0.18"`
candidate versions found which didn't match: 0.20.0
required by package `gtk v0.18.2`
    ... which satisfies dependency `gtk = "^0.18"` (locked to 0.18.2) of package `tauri v2.11.3`
```

Root cause: the **gtk-rs GTK3 bindings are frozen at the 0.18 series and are themselves
flagged unmaintained** — the same excluded workspace surfaces the entire GTK3-EOL cluster
(**RUSTSEC-2024-0411 … RUSTSEC-2024-0420**, "gtk-rs GTK3 bindings — no longer maintained",
over `atk` / `gdk` / `gdk-pixbuf` / `gtk` / `gtk3-macros` / …). `glib 0.20` is the **GTK4**
generation of gtk-rs; there is no GTK3 `gtk` release that depends on `glib 0.20`, so pulling
`glib` forward would require moving the whole webview stack to GTK4 / `webkitgtk-6.0`.

Neither is reachable from a Tauri **2.x** minor bump: `cargo update -p tauri` moves
`tauri 2.11.3 → 2.11.5` (latest 2.x) but touches only `tauri` + `tauri-runtime-wry` — the
`gtk 0.18.2` / `glib 0.18.5` pin is unchanged. The GTK4 migration is a Tauri-roadmap / gtk-rs
concern, not an in-repo lockfile edit. **A lockfile-only fix is therefore impossible**; forcing
it (per the bead) was deliberately NOT done.

## Reachability / exploitability (honest, caveated)

The unsoundness is in the `Iterator` impls of `glib::VariantStrIter`, a specific GLib
`Variant`-string-array iteration API. `sparq-gui` does **not** call `VariantStrIter` directly;
the crate is present transitively under Tauri's GTK3 event/IPC plumbing. This is **not** a
default-path memory-corruption bug — it requires the specific unsound iterator to be driven
over an attacker-shaped `Variant`. That said, this is a **best-effort reachability argument, not
an audited "not exploitable" verdict** — treat the desktop GUI as carrying a tolerated,
upstream-blocked medium unsoundness until the GTK4 migration lands. The GUI is an **opt-in**
surface built only in the path-scoped `.github/workflows/gui.yml` lane; it is not part of the
`sparq-core` / `sparq-server` library or engine attack surface that the certification
frameworks scope.

## Enforcement status (no in-repo gate today — a real gap)

- The **root** `cargo-deny` advisory gate does not scan this excluded workspace, so it neither
  fails nor suppresses this advisory. The root `deny.toml [advisories].ignore` list and
  `supply-chain/vex.cdx.json` stay **1:1 and empty** (enforced by
  `scripts/check-vex-deny-drift.py`) — this glib disposition is deliberately **NOT** added
  there, because that gate governs a different dependency graph (adding it would create an
  unmatched ignore / break the drift invariant).
- Note that even if this workspace were scanned by `cargo-deny` with its default policy,
  RUSTSEC-2024-0429 is an `unsound` **informational** advisory that the default policy does not
  fail on — cargo-deny surfaces the GTK3-EOL `unmaintained` cluster and unrelated
  vulnerabilities over this lock, but not 0429. Dependabot (GitHub advisory DB) is the surface
  that raises it as a security alert.
- **Follow-up (tracked below):** whether to add a gui-scoped advisory gate is a larger
  governance decision — it would have to tolerate the full GTK3-EOL cluster (~19 advisories on
  this lock), so it is intentionally out of scope for this single-alert disposition and is left
  as tracked follow-up rather than silently mass-suppressed.

## Tracking

- Dependabot alert **#5** (`repos/jeswr/sparq/dependabot/alerts/5`, state `open`). Its true
  resolution is the upstream **Tauri Linux GTK4 / `webkitgtk-6.0` migration** (which brings
  `gtk-rs 0.20` → `glib 0.20`); until then the alert may be dismissed on GitHub as
  *tolerable-risk / no fix available in the supported version line* (a maintainer decision, not
  an in-repo edit) or left open as a visible reminder.
- Upstream signal: the RustSec GTK3-EOL cluster (RUSTSEC-2024-0411…0420) is the authoritative
  "GTK3 is end-of-life" marker; Tauri's Linux backend consuming `webkit2gtk` (GTK3) is the
  binding constraint.
- In-repo tracking bead: **sq-x3r9b** (this record) + upstream-migration follow-up **sq-tuqht**
  (Tauri 2 Linux GTK4 migration; the true resolution of this and the GTK3-EOL cluster).
