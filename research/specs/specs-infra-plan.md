# /specs Infrastructure Plan

> Grounding for the specs infrastructure work (bead sq-rvgr2.1). Read-only recon; not itself normative.

## Summary

The `/papers` pipeline is a Typst→PDF+HTML build-time factory. Sources live in `site/papers/*.typ`,
a prebuild script (`site/scripts/build-papers.mjs`) compiles each `.typ` to a PDF artifact
(`site/public/papers/<slug>.pdf`, git-ignored) and a body-fragment HTML
(`site/src/generated/papers/<slug>.html`, git-ignored), and the Next.js static route
(`site/src/app/papers/[slug]/page.tsx`) reads the fragment via `readFileSync` at Next.js build
time and injects it via `dangerouslySetInnerHTML` — no runtime JS or WASM compiler shipped.

A `/specs` section can be built as an exact parallel, but the source compiler for specs is W3C
ReSpec (HTML→baked HTML via puppeteer) rather than Typst, which introduces a Chromium dependency
in the `pages.yml` CI workflow that currently has none.

---

## Papers Pipeline Mechanics (Exact)

### Source layout

- `site/papers/*.typ` — Typst sources; shared helper `site/papers/_lib/bench.typ` (evidence
  injection + honesty gate accessors `headline()`/`ev()`).
- `site/src/data/paper-evidence.json` — records with `value`/`source`/`environment`
  (`canonical|indicative`); the honesty gate in `build-papers.mjs:85–113` rejects any record
  that is not canonical/indicative or lacks `source`/`value`.
- `site/src/data/papers.ts` — exports `PAPERS: Paper[]` (slug, source, title, blurb, authors,
  venue, status, family, evidence). `build-papers.mjs` reads slug and source from this file via
  two regexes at lines 71–72 (no TS compiler needed); the script is plain `.mjs`.

### Build script `site/scripts/build-papers.mjs` (invoked as `prebuild` step)

1. `runHonestyGate()` — validates evidence JSON schema; exits 1 on bad records.
2. `runBuildBoundaryHonestyScan()` — re-runs `scripts/check-no-perf-numbers.py --enforce` +
   `scripts/check-privacy-claims.sh` (from `REPO_ROOT`) over the exact paper `.typ` sources +
   evidence file; FAIL-CLOSED.
3. `resolveTypst()` — `PATH || ~/.local/bin/typst || ~/.cargo/bin/typst`; returns null if
   absent.
4. Per paper: `execFileSync(typst, ["compile", typPath, pdfOut, "--root", SITE, "--input",
   "data=<evidenceJson>"])` → `public/papers/<slug>.pdf` (`PDF_OUT_DIR = site/public/papers/`).
5. Per paper: `execFileSync(typst, ["compile", typPath, htmlOut, "--format","html",
   "--features","html", "--root", SITE, "--input", "data=..."])` → temp full HTML file.
6. Regex-extracts `<body>…</body>` innerHTML from the full HTML (line 199–200); prepends a
   provenance HTML comment (`generatedHeader()`, lines 39–44); writes to
   `site/src/generated/papers/<slug>.html` (`HTML_OUT_DIR`).
7. Graceful fallback (lines 224–242): if typst absent, writes a
   `<p class="paper-placeholder">…</p>` so next build does not hard-fail; this is dev-only —
   CI always has typst.

### Evidence binding

Every number in a `.typ` comes only through `site/papers/_lib/bench.typ` — `headline(key)`
(line 31) PANICS the compile if the record's `environment != "canonical"`; `ev(key)` (line 28)
is the ungated accessor for indicative callouts. Same JSON feeds PDF and HTML so they cannot
disagree.

### Prebuild hook

`site/package.json` line 14: `"prebuild": "node scripts/sync-wasm.mjs && … && node
scripts/build-papers.mjs"`. Both `"build"` (via prebuild) and `"dev"` scripts trigger it.

### CI (`pages.yml`)

Installs Typst v0.15.0 pinned by SHA256 `59b207df…` (lines 165–175), then runs
`npm ci && npm run build` in `site/`.

### Next.js route `site/src/app/papers/[slug]/page.tsx`

- `generateStaticParams()` returns `PAPERS.map(p => ({slug: p.slug}))` — all slugs pre-rendered
  at build time.
- `dynamicParams = false` (line 35).
- `readPaperHtml(slug)` calls `readFileSync(join(process.cwd(), "src","generated","papers",
  <slug>.html))` at Next.js BUILD TIME (not at runtime; this is an RSC server-side read at the
  static-export generation step).
- Renders via `<PaperHtml html={html} />` which is `<article className="paper-prose"
  dangerouslySetInnerHTML={{__html: html}} />` (`site/src/components/papers/paper-html.tsx`).

### Navigation

`TOP_PAGES[]` entry `{href:"/papers", title:"Papers", blurb:"…"}` at
`command-palette.tsx:105` — Cmd-K accessible. NOT in `NAV_ITEMS[]` in `app-shell.tsx` (slim
bar kept at 6 destinations: Home/Capabilities/Try/App/Benchmarks/Download).

### Gitignore

`site/.gitignore` line 14 `/public/papers/` (PDF artifacts); line 15 `/src/generated/`
(HTML fragments). Both are regenerable build outputs, not tracked source.

---

## Recommended ReSpec Approach for `/specs`

**Option A — ReSpec CLI baked at build time (RECOMMENDED)**: This is an exact parallel of the
papers pipeline. The `respec` npm package (v37.1.0, devDependency) provides a CLI that uses
puppeteer (v24) to run ReSpec headlessly. The build script calls:

```sh
respec --src file:///abs/path/to/spec.html --out /tmp/spec-full.html \
  --localhost --use-local --haltonerror --no-sandbox
```

- `--use-local` avoids the W3C CDN fetch during build (network-independent).
- `--localhost` starts a local HTTP server so puppeteer loads a proper `http://` URL.
- `--no-sandbox` is required for headless Chrome in CI without user-namespace support.

After compilation: extract `<body>` innerHTML (same regex as `build-papers.mjs:199–200`),
prepend a provenance comment, write to `site/src/generated/specs/<slug>.html`. The Next.js
route reads the fragment at build time via `readFileSync`, injects via `dangerouslySetInnerHTML`
inside an `<article className="spec-prose">` wrapper. Static, SEO-indexable, no runtime JS.

**Option B — ReSpec client-side (CDN script) — NOT RECOMMENDED**: Page content is not rendered
in the static HTML, so web crawlers and users without JS see unformatted content. Conflicts with
the site's static-export philosophy.

**Option C — Bikeshed (no Chrome required)**: `python3 -m bikeshed spec site/specs/<slug>.bs
/tmp/<slug>.html`; fast (<2s per spec), no browser required, Python already a CI dep. Source
format `.bs` is less familiar to the Semantic Web/RDF community than ReSpec HTML. Viable if
Chrome in CI is a hard constraint.

**Overall recommendation**: Option A for correctness to the bead spec ("W3C ReSpec-style"); the
Chrome overhead in `pages.yml` is real but manageable. If Chrome is a hard constraint, use
Option C.

---

## Ordered Build Steps (15 steps)

1. Create `site/specs/` directory and write the first spec source as a standard ReSpec HTML
   file (e.g., `site/specs/sparql-extension-registry.html` with `respecConfig` JSON in a
   `<script class='remove'>`, `specStatus: 'unofficial'`, `shortName`, `editors` list).

2. Create `site/src/data/specs.ts` mirroring `papers.ts` shape: export `Spec` interface
   (slug: string, source: string, title: string, blurb: string, shortName: string, status:
   `'unofficial' | 'cg-draft' | 'draft'`, editors: string), export `SPECS: Spec[]`, export
   `specBySlug()`. The build script will regex-parse `slug+source` from this file exactly as
   `build-papers.mjs` reads `papers.ts`.

3. Create `site/scripts/build-specs.mjs` modelled on `build-papers.mjs`: (a) `readRegistry()`
   via regex over `specs.ts`; (b) `resolveRespec()` — check `node_modules/.bin/respec` (works
   after `npm ci` installs the devDep); (c) graceful fallback: if respec absent, write
   placeholder HTML to `src/generated/specs/<slug>.html` and return; (d) per spec: shell out
   to `respec --src file:///abs/path/to/spec.html --out /tmp/<slug>-full.html --localhost
   --use-local --haltonerror --no-sandbox`; (e) extract `<body>` innerHTML; (f) write
   provenance comment + fragment to `site/src/generated/specs/<slug>.html`.

4. Add `respec@37.1.0` as a `devDependency` in `site/package.json` (pin the version; avoids
   an `npx` download at build time and ensures the version is locked in the workspace
   lockfile).

5. Append `&& node scripts/build-specs.mjs` to the `prebuild` script in `site/package.json`
   (after `build-papers.mjs`). Also add it to the `dev` script so local dev generates
   placeholders.

6. Add `/public/specs/` to `site/.gitignore` (if any downloadable HTML spec artifacts are
   produced alongside the fragment; the existing `/src/generated/` entry at line 15 already
   covers `src/generated/specs/`).

7. Create `site/src/app/specs/layout.tsx`: identical to `papers/layout.tsx` —
   `<div className='mx-auto w-full max-w-3xl'>{children}</div>`.

8. Create `site/src/app/specs/page.tsx`: data-driven index card grid from `SPECS[]`, with
   status badges and links to `/specs/<slug>`. Mirror the card structure of `papers/page.tsx`.

9. Create `site/src/app/specs/[slug]/page.tsx`: `generateStaticParams()` from `SPECS`;
   `dynamicParams = false`; `readSpecHtml(slug)` via `readFileSync(join(process.cwd(),
   'src','generated','specs', slug+'.html'))` at build time; render via `<SpecHtml html={html}
   />`.

10. Create `site/src/components/specs/spec-html.tsx`:
    `<article className='spec-prose' dangerouslySetInnerHTML={{__html: html}} />`
    (mirrors `paper-html.tsx`).

11. Add `.spec-prose` CSS block to `site/src/app/globals.css` covering the ReSpec-generated
    class names that `.paper-prose` does not: `dfn { font-weight:600; cursor:help }`,
    `.note`/`.informative` (styled aside boxes), `.issue` (warning-coloured aside), `.example`
    (code-sample aside), `pre.idl` (monospace, border-left accent), `#toc ol
    { list-style:decimal; padding-left:1.5rem }`, `figure.syntax`, `.respec-dfn-list`. Reuse
    the existing OKLCH palette tokens (`--muted`, `--border`, `--primary`) for consistency.

12. Add `{ href: '/specs', title: 'Specs', blurb: 'W3C Unofficial Proposal Draft specifications
    produced by the sparq project.' }` to `TOP_PAGES[]` in
    `site/src/components/command-palette.tsx` (after the papers entry at line 105). Do NOT add
    it to `NAV_ITEMS[]` in `app-shell.tsx` — `/specs` stays Cmd-K accessible, keeping the slim
    bar at 6.

13. Update `pages.yml`: add a 'Install Chromium for respec' step before 'Build static site':
    `sudo apt-get install -y chromium-browser`; set env var
    `PUPPETEER_EXECUTABLE_PATH=$(which chromium-browser)` (or pass it to the `npm run build`
    env). Optionally cache the apt package or use `actions/cache` keyed on respec version to
    avoid repeated downloads.

14. Add a unit test or stub in `site/test/` that verifies the specs registry round-trips (slug
    uniqueness, source file existence check) — mirrors the pattern that `build-papers.mjs` uses
    for papers.

15. Smoke-test the assembled artifact in `pages.yml`: add `out/specs/` to the existing smoke-
    check step — assert `out/specs/index.html` exists and at least one `out/specs/<slug>/
    index.html` is present after the build.

---

## Risks

1. **Chrome/puppeteer not in `pages.yml`**: respec v37 depends on puppeteer v24 which needs
   Chromium. Adding `sudo apt-get install -y chromium-browser` adds ~60s; `npx playwright
   install --with-deps chromium` adds ~3min. Must also set `PUPPETEER_EXECUTABLE_PATH` so
   respec's puppeteer finds the system browser, otherwise puppeteer tries to download its own
   Chrome (~120MB) into `~/.cache/puppeteer` which may race with CI disk quotas.

2. **Fragment CSS surface area**: Typst HTML export emits clean `h2/h3/p/table/strong/em/code`
   semantic tags that `.paper-prose` handles with ~40 lines of CSS. ReSpec output uses ReSpec-
   specific class names (`dfn`, `.note`, `.issue`, `.example`, `.informative`, `.normative`,
   `pre.idl`, `figure.syntax`, `#toc`, `#toc ol`, `.respec-dfn-list`, `.respec-ref-bibl`)
   that require a substantially larger `.spec-prose` block. If the CSS block is incomplete,
   spec pages will render as unstyled HTML inside the sparq site shell.

3. **lychee `--include-fragments` regression**: ReSpec bakes a dense table of contents with
   `href='#section-N-title'` anchors. The `pages.yml` link-check (`check-site-links.sh`,
   `lychee --include-fragments`) validates every `#fragment`. If a ReSpec-generated spec
   section ID does not match a link (e.g., a manually-written cross-reference inside the spec
   that points to a section that was renumbered), the link-check fails the entire `pages.yml`
   build. Test locally before merging.

4. **respec `--localhost` port collision**: `build-specs.mjs` runs one respec process per spec;
   each `--localhost` invocation starts a local HTTP server on the default port 3000. If two
   specs are compiled in parallel (or if the build environment has another service on 3000),
   the port conflicts. The `--port` flag can override; process specs sequentially and pass a
   distinct `--port` per invocation.

5. **ReSpec `--use-local` CDN independence**: without `--use-local`, respec fetches the W3C
   CDN at render time. In CI this depends on outbound network availability and CDN uptime. The
   `--use-local` flag MUST be set to make builds deterministic and network-independent.

6. **TypeScript and eslint gates**: the new `specs.ts`, route pages, and `spec-html.tsx`
   component must pass the `eslint-config-next` typecheck that runs as part of `npm run lint`
   (`site-e2e.yml` lint step). Any unused import, missing return type annotation, or missing
   `'use client'` directive on a client component will fail the lint gate.

7. **Nav test brittleness**: `site-e2e.yml` runs `site-nav.spec.ts` which asserts the slim bar
   has exactly 6 destinations. As long as `/specs` stays in Cmd-K only (`TOP_PAGES`, not
   `NAV_ITEMS`), this test is unaffected.

8. **Placeholder fallback must match both formats**: `build-specs.mjs` must write a placeholder
   HTML when respec is absent (following the existing paper placeholder pattern
   `<p class='paper-placeholder'>…</p>`) so `readFileSync` at Next.js build time does not
   produce a broken `dangerouslySetInnerHTML` injection.

---

> **Empirical-honesty reminder**: ZK and MPC estates are NOT production-sound until the
> external cryptographer audit sq-qhy4 completes. All work-box benchmarks are non-canonical;
> do not hard-code them in documentation or tests.

---

*Recon captured by Sonnet 4.6 under the Fable program; [SONNET-4.6]*
