#!/usr/bin/env node
// [OPUS-4.8] sq-jpki.1 — derive a SELF-CONTAINED, standalone npm lockfile for ONE
// workspace member out of the repo-root `package-lock.json`, WITHOUT a network install.
//
// WHY: after the repo-root npm-workspaces migration there is a SINGLE root
// `package-lock.json` (the per-member locks were dropped as redundant). But
// `scripts/gen-js-sbom.sh` runs `cyclonedx-npm --package-lock-only` from `js/`, which
// requires a lockfile co-located with `js/package.json` AND must NOT hit the network (the
// supply-chain `js-sbom` lane is deliberately install-free + deterministic). This script
// projects the member's dependency CLOSURE out of the committed root lock into a temporary
// standalone lockfile whose root component is the member itself (e.g. `@jeswr/sparq`), so
// the SBOM stays anchored at the published client — exactly as it did pre-migration with a
// committed per-package lock — while preserving the EXACT versions the root lock pins (no
// `^`-range re-resolution, no registry call). The emitted lock is TRANSIENT scratch (the
// caller removes it); it is never committed (the root lock is the single source of truth).
//
// Usage:  node scripts/derive-workspace-member-lock.mjs <member-dir> <out-lock-path>
//   e.g.  node scripts/derive-workspace-member-lock.mjs js js/package-lock.json
//
// Assumptions (asserted): the root lock is lockfileVersion 3 and every dependency in the
// member's closure is hoisted to the root `node_modules/<name>` (npm's default). If a dep
// were NESTED under `<member>/node_modules/<name>` (a version conflict), this script also
// looks there and re-homes it to top-level in the standalone lock; an UNRESOLVED dep is a
// hard error (so a future hoisting change fails LOUDLY rather than silently dropping a
// component from the SBOM).
import fs from "node:fs";
import path from "node:path";

const [, , memberDir, outPath] = process.argv;
if (!memberDir || !outPath) {
  console.error("usage: derive-workspace-member-lock.mjs <member-dir> <out-lock-path>");
  process.exit(2);
}

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const rootLockPath = path.join(repoRoot, "package-lock.json");
if (!fs.existsSync(rootLockPath)) {
  console.error(`ERROR: root lockfile not found at ${rootLockPath}`);
  process.exit(1);
}
const root = JSON.parse(fs.readFileSync(rootLockPath, "utf8"));
if (root.lockfileVersion !== 3) {
  console.error(`ERROR: expected lockfileVersion 3, got ${root.lockfileVersion}`);
  process.exit(1);
}

const memberKey = memberDir.replace(/\/+$/, "");
const memberPkg = root.packages[memberKey];
if (!memberPkg) {
  console.error(`ERROR: workspace member "${memberKey}" not found in root lock packages map`);
  process.exit(1);
}

const declared = (pkg) =>
  Object.keys({
    ...(pkg.dependencies || {}),
    ...(pkg.devDependencies || {}),
    ...(pkg.optionalDependencies || {}),
  });

// Resolve a dependency name to its lock key, preferring a member-nested entry, else top-level.
const resolveKey = (name) => {
  const nested = `${memberKey}/node_modules/${name}`;
  if (root.packages[nested]) return { key: nested, node: root.packages[nested] };
  const top = `node_modules/${name}`;
  if (root.packages[top]) return { key: top, node: root.packages[top] };
  return null;
};

const out = {
  name: memberPkg.name,
  version: memberPkg.version,
  lockfileVersion: 3,
  requires: true,
  packages: {},
};
// The standalone lock's root ("") IS the member — this anchors cyclonedx's root component.
out.packages[""] = { ...memberPkg };

const queue = declared(memberPkg).map((n) => n);
const seen = new Set();
let unresolved = 0;
while (queue.length) {
  const name = queue.shift();
  if (seen.has(name)) continue;
  seen.add(name);
  const r = resolveKey(name);
  if (!r) {
    console.error(`ERROR: dependency "${name}" in the ${memberKey} closure is not present in the root lock`);
    unresolved++;
    continue;
  }
  // Re-home to a top-level node_modules/<name> key in the standalone lock (flat, hoisted).
  out.packages[`node_modules/${name}`] = r.node;
  for (const d of declared(r.node)) queue.push(d);
}
if (unresolved > 0) {
  console.error(`ERROR: ${unresolved} dependency(ies) could not be resolved from the root lock`);
  process.exit(1);
}

fs.mkdirSync(path.dirname(path.resolve(outPath)), { recursive: true });
fs.writeFileSync(outPath, `${JSON.stringify(out, null, 2)}\n`);
console.error(
  `derived standalone lock for "${memberKey}" (${memberPkg.name}@${memberPkg.version}) -> ${outPath}: ${Object.keys(out.packages).length - 1} dependency package(s)`,
);
