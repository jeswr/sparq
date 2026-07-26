# Visual-regression suite (sq-ymr2e.10)

Design of record: `research/web-gui-test-program.md` §4. Playwright `toHaveScreenshot`
over the per-PR **key layouts** (home first fold dark + one light shot,
`/download` in the release-fixture state, the mpc-100k showcase,
nav + palette-open overlay), at two pinned viewports (1280×720 `visual-desktop`,
390×844 `visual-mobile`). The full-surface sweep is the nightly lane (sq-ymr2e.11).

## The container rule (why your laptop must not mint baselines)

Screenshots are stable only when the font rasterizer / antialiasing / browser build are
pinned, so this suite runs **only inside the digest-pinned official Playwright container**
(`scripts/vr.sh`) and the committed baselines under `baselines/` are **Linux-only by
policy** — there is no per-OS baseline zoo. The `visual-*` projects exist only under
`SPARQ_VR=1` (set by `vr.sh`), and the config refuses `SPARQ_VR=1` outside the container.

```sh
# prereqs: repo-root `npm ci`; the lean wasm bundle in site/public/wasm
#   (cd ../js && npm run build:wasm:lean) && npm run sync-wasm
npm run vr           # compare against the committed baselines
npm run vr:update    # regenerate every baseline (forces a rewrite)
```

## Updating baselines (the reviewed-artifact workflow)

An **intentional** visual change refreshes its baselines **in the same PR**: run
`npm run vr:update`, eyeball the changed PNGs in the diff, and commit them alongside the
code change. An **unexpected** failure uploads readable diff artifacts
(`test-results/**` — `*-actual.png` / `*-expected.png` / `*-diff.png`) in the advisory
`site-visual` CI lane; treat it as a real regression until shown otherwise.

Bumping `@playwright/test` = re-pin the image tag **and digest** in `scripts/vr.sh` and
regenerate all baselines in the same PR (the browser build changes the rasterization).

### No docker? Re-mint the baselines in CI (sq-hfd82)

`vr:update` needs the pinned image, so a machine without a docker daemon cannot refresh
baselines at all — which is how the nightly `visual-sweep` ends up reporting drift that
nobody in that position can clear. Recovery path: dispatch **`nightly-full-sweep`** with
`mode: full` **and** `refresh_visual_baselines: true`. That run skips the comparison, runs
`vr:update` inside the same pinned container, and uploads the regenerated PNGs as the
`refreshed-visual-baselines` artifact; failure routing is forced off so the consolidated
visual issue is not auto-closed before the refresh has actually landed. Download it,
**eyeball every changed image** — a refresh blesses whatever rendered, a real regression
included — and commit the PNGs in a PR.

The nightly full-surface baselines (`full-surface.spec.ts`) drift by design: that spec runs
only under `SPARQ_NIGHTLY_VR`, so PRs that restyle the app shell or a captured page never see
their own pixel change. Expect accumulated intentional churn there, and reserve the
"regression until shown otherwise" reading for the per-PR key layouts.

## Dynamic content: mask, don't chase

Any region whose pixels are data-driven (measured `ms` timings, release version/size/sha
strings, benchmark numbers, paper dates) carries a `data-vr-mask` attribute in the
component and is masked at capture, so baselines survive content churn. When a new
dynamic region appears, add `data-vr-mask` to it — never loosen the comparator.

A mask hides a region's **pixels**, not its **geometry**: Playwright paints it over the
element's LIVE bounding box, so a masked region that RESIZES with its data still moves the
pixels around it and the baseline drifts anyway. Where the data can change the region's size
(a digit more, a singular/plural label, a longer version string), give it a data-independent
box too — `src/lib/metric-badge.ts` is the worked example: it reserves the widest label the
pill can ever render as an invisible ghost and centres the live label over it. `tabular-nums`
is NOT sufficient; it equalises digit advance widths only, not total label widths.

The lane is **advisory** (§6.3) and must earn gating separately: 50 consecutive green
runs spanning ≥10 distinct PRs or two weeks, whichever is longer.
