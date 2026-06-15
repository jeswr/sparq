<!-- [OPUS-4.8] sq-9xgq -->
# Vendored frontend assets (`bench/dashboard/vendor/`)

These files are third-party libraries checked into the repo **on purpose**, so the
benchmark dashboard has **no `cdn.jsdelivr.net` runtime dependency**. Serving them from the
repo (and from the `dev/bench/` copy on the `benchmark-data` branch) is a supply-chain +
offline-resilience improvement, and a prerequisite for the unified GitHub Pages cutover.

This mirrors the repo's "SHA-pin with a readable comment" convention (see how GitHub Actions
are pinned by SHA): the exact version + source + integrity hash are recorded below so future
bumps are legible and verifiable.

> **Publishing follow-up (out of scope for sq-9xgq).** The local copy works when the dashboard
> is opened from this directory. For the **published** Pages dashboard, the `Seed Pages dashboard
> onto benchmark-data` step in `.github/workflows/bench.yml` currently copies a fixed flat file
> list (`index.html dashboard.js dashboard.css metric-labels.json`) into `dev/bench/` — it does
> **not** yet copy `vendor/Chart.min.js`, so the live page would 404 on the script until that step
> learns to copy this subdirectory (add `vendor/Chart.min.js` to the loop, `mkdir -p` the
> per-file dirname, and add it to the final `git add`). That workflow change lives outside
> `bench/dashboard/` and is left as a separate task.

## `Chart.min.js`

| field        | value |
| ------------ | ----- |
| library      | [Chart.js](https://www.chartjs.org/) |
| version      | **2.9.2** (pinned; the same version the dashboard previously loaded from the CDN) |
| license      | MIT |
| source URL   | `https://cdn.jsdelivr.net/npm/chart.js@2.9.2/dist/Chart.min.js` |
| sha256       | `6485aa93c81317de6df661c89711cbe32718bb9d881d5703884f6be566ae3631` |
| size         | 172810 bytes |
| referenced by| `../index.html` (`<script src="vendor/Chart.min.js">`), used by `../dashboard.js` |

> Note: Chart.js 2.x ships its UMD build as `Chart.min.js` (capital C). The `chart.umd.min.js`
> filename only exists for Chart.js 3.x and later — do not rename this file when re-vendoring 2.x.

## How to re-vendor (bump the version)

1. Pick the new version `X.Y.Z` and fetch the matching dist file from jsDelivr:

   ```sh
   # Chart.js 2.x:
   curl -fsSL "https://cdn.jsdelivr.net/npm/chart.js@X.Y.Z/dist/Chart.min.js" \
     -o bench/dashboard/vendor/Chart.min.js
   # Chart.js 3.x+ uses a different dist path/name (chart.umd.min.js) and a different API —
   # treat a major bump as a code change to dashboard.js, not a drop-in swap.
   ```

2. Record the new version, source URL, byte size, and integrity hash here:

   ```sh
   sha256sum bench/dashboard/vendor/Chart.min.js
   wc -c     bench/dashboard/vendor/Chart.min.js
   ```

3. If the filename changes (e.g. moving to `chart.umd.min.js` on a 3.x bump), update the
   `<script src="...">` reference in `../index.html` **and** the file list in the
   `Seed Pages dashboard onto benchmark-data` step of `.github/workflows/bench.yml`, so the
   published `dev/bench/` copy picks it up.

4. Open the dashboard locally (`open bench/dashboard/index.html`, or serve the dir) and
   confirm the trend / scaling charts still render.
