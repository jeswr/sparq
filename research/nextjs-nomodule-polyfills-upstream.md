# Upstream Next.js: config seam to omit the legacy `noModule` polyfills chunk

**Bead:** sq-zv37m · **Status:** draft prepared, NOT yet proposed upstream — awaiting @jeswr review per the
upstream-contribution protocol · **Author:** SPARQ agent 🤖 [FABLE-5] · **Date:** 2026-07-12

## Problem

Next.js unconditionally ships a pre-compiled legacy-browser polyfills chunk
(`_next/static/chunks/polyfills-*.js`) with every client build — including `output: "export"` — loaded
via `<script … noModule>`. In sparq's GUI export (next 15.5.x, 2026-07-12) it measured **112,594 B raw /
39,509 B gzip**. An ES-module-capable engine never fetches a `noModule` classic script, and the Tauri 2
webview (WebView2 / WKWebView / WebKitGTK) is always module-capable, so for the desktop bundle the chunk
is pure dead installer weight. There is **no config to omit it**.

- **sparq interim mitigation:** PR #2016 — a post-export strip wired into `gui/app` `build:tauri` only
  (hard-errors when the chunk disappears, so the step is deleted rather than rotting once this upstream
  seam lands).
- **This record:** the prepared minimal upstream patch for vercel/next.js.

## Where Next.js injects the chunk (canary, `next@16.3.0-canary.83`)

| Site | Role |
|---|---|
| `packages/next/src/build/webpack-config.ts:2176` | `isClient && new CopyFilePlugin({ filePath: require.resolve('./polyfills/polyfill-nomodule'), name: 'static/chunks/polyfills[-hash].js', info: { [CLIENT_STATIC_FILES_RUNTIME_POLYFILLS_SYMBOL]: 1 } })` — the single choke point: the pre-compiled asset is *copied* into the client build (it is not a webpack entry) |
| `packages/next/src/build/webpack/plugins/build-manifest-plugin.ts:172` | `assetMap.polyfillFiles` = compilation assets carrying the symbol marker |
| `packages/next/src/pages/_document.tsx:67` (`getPolyfillScripts`) | pages-router render: `buildManifest.polyfillFiles` → `<script noModule>` |
| `packages/next/src/server/app-render/app-render.tsx:3281, 7742` | app-router render (both render paths): same mapping |
| Turbopack `crates/next-api/src/app.rs:1405-1456` | separate Rust-side emission (app router only) — **not covered by this patch** |

Empty `polyfillFiles` ⇒ zero `noModule` tags in **both** routers automatically — no render-side change
needed. Checked against `v15.5.9` (what sparq consumes): the mechanism is byte-for-byte identical, so the
patch backports trivially.

## Chosen seam

`experimental.legacyBrowserPolyfills?: false` — a verbatim copy of the existing
`experimental.fallbackNodePolyfills?: false` convention (type `?: false` in `config-shared.ts`,
`z.literal(false).optional()` in `config-schema.ts`, consumed via `!== false`, no defaultConfig entry).
When `false`, webpack skips the `CopyFilePlugin` → no chunk on disk → `polyfillFiles === []` → no
`noModule` script in either router, dev and prod, including static export. Flag absent ⇒ behaviour
byte-identical to today.

**Turbopack:** honestly out of scope — registered in `turbopack-warning.ts`'s
`unsupportedTurbopackNextConfigOptions` so `--turbopack` warns instead of silently ignoring (same as
`fallbackNodePolyfills`). Vercel's own open PR #88551 already moves Turbopack to browserslist-driven
polyfill selection; this flag is the minimal webpack-side opt-out that composes with (and could later be
superseded by) that approach.

## Prior art (searched before drafting)

- vercel/next.js **#86785** (issue, open) — "Polyfills for unsupported browsers are enabled by default";
  the canonical, heavily-upvoted complaint (Lighthouse flags the chunk on every default app).
- **#88551** (PR, open, Vercel maintainer) — Turbopack-only browserslist-driven selection; does not touch
  the webpack path.
- **#87270** (PR, open, community) — shrinks the module-polyfill contents; superseded-listed by #88551.
- **#75820** (PR, open since 2025-02) — raise the default browserslist; stalled.
- **#10212 / #21418 / #27596 / #65223** (merged) — how the chunk got its CopyFilePlugin + symbol-marker
  + manifest shape.
- No existing issue/PR proposes a webpack-side config seam; no name collision for
  `legacyBrowserPolyfills` (Next 12's removed `experimental.legacyBrowsers` is adjacent naming precedent).

## Prepared branch + patch

Branch `feat/omit-legacy-polyfills` on the local clone (`/tmp/nextjs-upstream`, base `canary`); patch
archived below (5 files, +82/−0). Nothing pushed to vercel/next.js; no issue/PR opened there.

### Draft PR description (Next.js style)

> Next.js unconditionally ships a pre-compiled ~110 KB (raw) `polyfills-*.js` chunk loaded via
> `<script noModule>`, even though the default browserslist target is far past ES-module support —
> Lighthouse flags it on every default app (#86785). For deployments whose runtime is a known modern
> engine (e.g. an embedded webview consuming `output: "export"`), the chunk is pure dead weight in the
> shipped artifact and there is currently no way to opt out. This adds
> `experimental.legacyBrowserPolyfills: false`, following the exact shape of
> `experimental.fallbackNodePolyfills: false`: it skips the `CopyFilePlugin` that emits the chunk, which
> leaves `buildManifest.polyfillFiles` empty and therefore omits the `noModule` script tag in both
> routers with no render-side changes. Default behaviour is unchanged. Scope is webpack only; Turbopack
> warns via the unsupported-options list, and #88551 already moves Turbopack to browserslist-driven
> polyfill selection — this flag is the minimal webpack-side opt-out until webpack gets the same
> treatment.

### The diff

```diff
diff --git a/packages/next/src/build/webpack-config.ts b/packages/next/src/build/webpack-config.ts
--- a/packages/next/src/build/webpack-config.ts
+++ b/packages/next/src/build/webpack-config.ts
@@ -2174,6 +2174,7 @@ export default async function getBaseWebpackConfig(
         : new ProfilingPlugin({ runWebpackSpan, rootDir: dir }),
       new WellKnownErrorsPlugin(),
       isClient &&
+        config.experimental.legacyBrowserPolyfills !== false &&
         new CopyFilePlugin({
           // file path to build output of `@next/polyfill-nomodule`
           filePath: require.resolve('./polyfills/polyfill-nomodule'),
diff --git a/packages/next/src/lib/turbopack-warning.ts b/packages/next/src/lib/turbopack-warning.ts
--- a/packages/next/src/lib/turbopack-warning.ts
+++ b/packages/next/src/lib/turbopack-warning.ts
@@ -23,6 +23,7 @@ const unsupportedTurbopackNextConfigOptions = [
   'experimental.allowedRevalidateHeaderKeys',
   'experimental.extensionAlias',
   'experimental.fallbackNodePolyfills',
+  'experimental.legacyBrowserPolyfills',
 
   'experimental.swcTraceProfiling',
 
diff --git a/packages/next/src/server/config-schema.ts b/packages/next/src/server/config-schema.ts
--- a/packages/next/src/server/config-schema.ts
+++ b/packages/next/src/server/config-schema.ts
@@ -271,6 +271,7 @@ export const experimentalSchema = {
   imgOptSkipMetadata: z.boolean().optional().nullable(),
   isrFlushToDisk: z.boolean().optional(),
   largePageDataBytes: z.number().optional(),
+  legacyBrowserPolyfills: z.literal(false).optional(),
   linkNoTouchStart: z.boolean().optional(),
   manualClientBasePath: z.boolean().optional(),
   middlewarePrefetch: z.enum(['strict', 'flexible']).optional(),
diff --git a/packages/next/src/server/config-shared.ts b/packages/next/src/server/config-shared.ts
--- a/packages/next/src/server/config-shared.ts
+++ b/packages/next/src/server/config-shared.ts
@@ -688,6 +688,12 @@ export interface ExperimentalConfig {
    * [webpack/webpack#ModuleNotoundError.js#L13-L42](https://github.com/webpack/webpack/blob/2a0536cf510768111a3a6dceeb14cb79b9f59273/lib/ModuleNotFoundError.js#L13-L42)
    */
   fallbackNodePolyfills?: false
+  /**
+   * If set to `false`, the pre-compiled polyfills chunk for legacy browsers
+   * without ES module support (`polyfills-*.js`, loaded via `<script noModule>`)
+   * is omitted from the client build.
+   */
+  legacyBrowserPolyfills?: false
   sri?: {
     algorithm?: SubresourceIntegrityAlgorithm
   }
diff --git a/test/production/disable-legacy-browser-polyfills/index.test.ts b/test/production/disable-legacy-browser-polyfills/index.test.ts
new file mode 100644
--- /dev/null
+++ b/test/production/disable-legacy-browser-polyfills/index.test.ts
@@ -0,0 +1,73 @@
+import { nextTestSetup } from 'e2e-utils'
+import { readdirSync } from 'fs'
+import { join } from 'path'
+
+// TODO: Implement experimental.legacyBrowserPolyfills for Turbopack
+;(process.env.IS_TURBOPACK_TEST ? describe.skip : describe)(
+  'Disable legacy browser polyfills',
+  () => {
+    const { next } = nextTestSetup({
+      files: {
+        'app/layout.js': `
+          export default function Layout({ children }) {
+            return (
+              <html>
+                <body>{children}</body>
+              </html>
+            )
+          }
+        `,
+        'app/page.js': `
+          export default function Page() {
+            return <p>app router</p>
+          }
+        `,
+        'pages/legacy.js': `
+          export default function Page() {
+            return <p>pages router</p>
+          }
+        `,
+      },
+    })
+
+    function getPolyfillChunks() {
+      return readdirSync(
+        join(next.testDir, '.next', 'static', 'chunks')
+      ).filter((file) => file.startsWith('polyfills'))
+    }
+
+    async function getNoModuleScriptCount(pathname: string) {
+      const $ = await next.render$(pathname)
+      return $('script[nomodule]').length
+    }
+
+    it('emits the polyfills chunk and nomodule script by default', async () => {
+      const buildManifest = await next.readJSON('.next/build-manifest.json')
+      expect(buildManifest.polyfillFiles).not.toHaveLength(0)
+      expect(getPolyfillChunks()).not.toHaveLength(0)
+
+      expect(await getNoModuleScriptCount('/')).toBeGreaterThan(0)
+      expect(await getNoModuleScriptCount('/legacy')).toBeGreaterThan(0)
+    })
+
+    it('omits the polyfills chunk and nomodule script when disabled', async () => {
+      await next.stop()
+      await next.patchFile(
+        'next.config.js',
+        `module.exports = {
+          experimental: {
+            legacyBrowserPolyfills: false
+          }
+        }`
+      )
+      await next.start()
+
+      const buildManifest = await next.readJSON('.next/build-manifest.json')
+      expect(buildManifest.polyfillFiles).toHaveLength(0)
+      expect(getPolyfillChunks()).toHaveLength(0)
+
+      expect(await getNoModuleScriptCount('/')).toBe(0)
+      expect(await getNoModuleScriptCount('/legacy')).toBe(0)
+    })
+  }
+)
```

The test is modeled on `test/production/disable-fallback-polyfills/` (the sibling flag's test), skipped
under Turbopack with the customary `// TODO` marker, and asserts the default path first so the flag's
effect is proven non-vacuous.

## Risks / open questions for @jeswr

1. **Flag vs browserslist:** maintainers may prefer extending #88551's browserslist-driven behaviour to
   webpack rather than adding a flag. The draft PR body addresses this head-on; the guard is a one-line
   swap if they steer that way. An alternative is to comment on #86785/#88551 first and offer the
   webpack-side patch there.
2. **Naming:** `legacyBrowserPolyfills` (chosen for symmetry with `fallbackNodePolyfills` and Next 12's
   removed `experimental.legacyBrowsers`) vs e.g. `noModulePolyfills`.
3. **Where to target:** `canary` (verified) — backport to 15.5.x is byte-identical but upstream will not
   take 15.x PRs.

@jeswr — please review the seam + draft PR body above **before** anything is opened on vercel/next.js
(upstream-contribution protocol). On approval the next step is: push the branch to a fork, open the PR
tagging the relevant Lighthouse issue #86785, and cross-reference #88551.
