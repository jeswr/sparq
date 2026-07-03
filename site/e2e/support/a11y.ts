// [OPUS-4.8] sq-ymr2e.4 — the AUTOMATED accessibility gate for the site Playwright suite.
//
// Design of record: research/web-gui-test-program.md §3.1 — `@axe-core/playwright` pinned to the
// WCAG 2.1 A+AA rule tags, run against the P1 interactive surfaces IN THEIR KEY STATES (a11y bugs
// live in the OPEN palette / EXPANDED drawer / ERROR strip, not the pristine page load). This
// module is the shared gate the a11y spec (e2e/a11y.spec.ts) calls; the gate shape, the rule
// pinning, and the ratchet mechanics live here in ONE place.
//
// TWO GATE TIERS (both ratchet moderate/minor):
//   * ZERO gate (assertA11y default) — serious + critical violations = HARD FAIL AT ZERO from day
//     one. Unconditional; never read from the baseline, so the baseline can never be edited to
//     hide a serious bug on a clean surface.
//   * KNOWN-GAP guard (assertA11y with `knownGap`) — a surface with a STRUCTURAL serious violation
//     that needs a design-token fix (filed as a bead, per §3.1 "file per-surface fix beads where
//     structural") is NOT dropped: it is scanned with an ALLOWANCE of exactly the beaded serious
//     rule-ids. The guard FAILS on any NEW serious rule (a regression) — the pre-existing beaded
//     set may only SHRINK (re-seed as beads land), mirroring the coverage ratchet. Nothing is
//     hidden: every allowance is listed with its bead in bench/a11y-baseline.json.
//
// In BOTH tiers, moderate + minor counts are a CEILING in bench/a11y-baseline.json that may only
// FALL — the gate fails on any RISE. Color-contrast + form-label rules are explicitly guaranteed
// on (never excludable). Automated axe covers ~the mechanical half of WCAG; the keyboard/focus
// half is the sibling behavioural bead (sq-ymr2e.9). We therefore claim "zero known serious/
// critical automated violations at WCAG 2.1 AA" on the gated surfaces, never "WCAG-compliant".
import { AxeBuilder } from "@axe-core/playwright";
import { expect, type Page } from "@playwright/test";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { Result as AxeViolation } from "axe-core";

/** The WCAG 2.1 A + AA rule tags axe runs. Level A + AA only (AAA is not the site's bar, §3). */
export const WCAG_TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] as const;

/** Rules the design pins ON and this gate refuses to let the baseline exclude (§3.1). */
const NEVER_EXCLUDABLE = ["color-contrast", "label"] as const;

/** Axe severity buckets. A `null`-impact violation is counted as `minor` (the axe convention). */
export type Impact = "critical" | "serious" | "moderate" | "minor";
export type ImpactCounts = Record<Impact, number>;

/** A structural serious violation tracked as a bead rather than fixed in the harness PR (§3.1). */
export type KnownGap = {
  /** The fix bead (e.g. "sq-abc12") so the allowance is never a silent whitewash. */
  bead: string;
  /** One line: why this is beaded not fixed here (design-token change, needs the visual-reg net). */
  reason: string;
};

/** The committed ratchet file (bench/a11y-baseline.json), repo-root-relative from this module. */
export const BASELINE_PATH = fileURLToPath(
  new URL("../../../bench/a11y-baseline.json", import.meta.url),
);

/** When set (A11Y_SEED=1), the run MEASURES + rewrites the baseline instead of ratcheting — but
 *  the ZERO tier's serious bar is STILL enforced, so a seed can never bless a NEW serious bug. */
export const SEED = !!process.env.A11Y_SEED;

type SurfaceBaseline = ImpactCounts & {
  /** Present ⇒ this surface is a KNOWN-GAP guard; the listed serious rule-ids are the allowance. */
  allowedSeriousRules?: string[];
  bead?: string;
  reason?: string;
};

type Baseline = {
  wcagTags: string[];
  /** ruleId → human reason. Kept SMALL and justified; NEVER a serious/critical rule (§3.1). */
  ruleExclusions: Record<string, string>;
  /** surface:state key → the ratchet ceiling (+ known-gap allowance where applicable). */
  surfaces: Record<string, SurfaceBaseline>;
};

const BASELINE_COMMENT = [
  "[OPUS-4.8] A11Y RATCHET (sq-ymr2e.4) — automated axe-core WCAG 2.1 AA violation counts per P1",
  "interactive surface x UI state, reviewed in diffs exactly like the coverage-floor counts.",
  "GATE (e2e/support/a11y.ts): two tiers. ZERO-tier surfaces (no allowedSeriousRules) HARD-FAIL on",
  "any serious/critical (unconditional). KNOWN-GAP surfaces (allowedSeriousRules + bead) tolerate",
  "EXACTLY the listed, beaded serious rule-ids and FAIL on any NEW one — the allowance may only",
  "SHRINK. In both tiers moderate+minor are a CEILING that may only FALL. Color-contrast and form-",
  "label rules are explicitly on and cannot be excluded. Re-seed after a fix or a new state:",
  "A11Y_SEED=1 npm run test:e2e -- a11y.spec.ts  (needs the wasm bundle for the results/error",
  "states — synced from `(cd ../js && npm run build:wasm)`; the light CI lane has no wasm, so it",
  "scans only the wasm-free states and never trips a wasm-state ratchet). Counts are measured on a",
  "non-canonical work box / CI runner — a REGRESSION ratchet, not a canonical audit number, and a",
  "FLOOR not a certificate (axe covers ~the mechanical half of WCAG; the keyboard/focus half is",
  "sq-ymr2e.9). We claim 'zero known serious/critical automated violations at WCAG 2.1 AA' on the",
  "gated surfaces, never 'WCAG-compliant'.",
];

/** Load + validate the committed baseline (throws if a serious/critical rule was excluded). */
export function loadBaseline(): Baseline {
  const raw = JSON.parse(readFileSync(BASELINE_PATH, "utf8")) as Baseline & { _comment?: unknown };
  // Safe defaults so a baseline that omits these keys (e.g. a hand-edited or freshly-seeded file
  // that hasn't yet committed both sections) can't crash the harness at runtime. [SONNET-4.6]
  raw.ruleExclusions ??= {};
  raw.surfaces ??= {};
  const excluded = Object.keys(raw.ruleExclusions);
  for (const rule of NEVER_EXCLUDABLE) {
    if (excluded.includes(rule)) {
      throw new Error(
        `[a11y] baseline excludes "${rule}", which the design pins ON (§3.1). Remove the exclusion.`,
      );
    }
  }
  return raw;
}

const baseline = loadBaseline();

/** What each scanned state measured — accumulated so A11Y_SEED can rewrite the baseline in afterAll. */
type Measured = { counts: ImpactCounts; seriousRules: string[]; knownGap?: KnownGap };
const measured = new Map<string, Measured>();

/** An axe scanner pinned to the WCAG 2.1 A+AA tags, with any documented rule exclusions applied. */
function buildAxe(page: Page, include?: string): AxeBuilder {
  let builder = new AxeBuilder({ page }).withTags([...WCAG_TAGS]);
  const excluded = Object.keys(baseline.ruleExclusions);
  if (excluded.length > 0) builder = builder.disableRules(excluded);
  if (include) builder = builder.include(include);
  return builder;
}

/** Count violations by severity. Axe's `impact` can be null on an edge — bucket as minor. */
export function countByImpact(violations: AxeViolation[]): ImpactCounts {
  const counts: ImpactCounts = { critical: 0, serious: 0, moderate: 0, minor: 0 };
  for (const v of violations) counts[(v.impact as Impact | null) ?? "minor"] += 1;
  return counts;
}

/** The sorted, de-duplicated rule-ids of the serious+critical violations (the known-gap allowance). */
function seriousRuleIds(violations: AxeViolation[]): string[] {
  const ids = violations
    .filter((v) => v.impact === "serious" || v.impact === "critical")
    .map((v) => v.id);
  return [...new Set(ids)].sort();
}

/** A readable one-block-per-violation summary for a failure message (id · impact · help · targets). */
export function formatViolations(violations: AxeViolation[]): string {
  if (violations.length === 0) return "  (none)";
  return violations
    .map((v) => {
      const targets = v.nodes
        .slice(0, 4)
        .map((n) => `      ${Array.isArray(n.target) ? n.target.join(" ") : String(n.target)}`)
        .join("\n");
      return `  • [${v.impact ?? "minor"}] ${v.id}: ${v.help}\n    ${v.helpUrl}\n${targets}`;
    })
    .join("\n");
}

export type AssertA11yOptions = {
  /** CSS selector to SCOPE the scan (used by the non-vacuity probe). */
  include?: string;
  /** Mark this state a KNOWN-GAP guard: tolerate exactly its beaded serious rule-ids (may shrink). */
  knownGap?: KnownGap;
};

/**
 * Run the axe WCAG 2.1 AA scan on the current page state and enforce the gate for `key`
 * (a "surface:state" label, e.g. "download:release"). Returns the measured counts.
 */
export async function assertA11y(
  page: Page,
  key: string,
  opts: AssertA11yOptions = {},
): Promise<ImpactCounts> {
  const { violations } = await buildAxe(page, opts.include).analyze();
  const counts = countByImpact(violations);
  const observedSerious = seriousRuleIds(violations);
  measured.set(key, { counts, seriousRules: observedSerious, knownGap: opts.knownGap });

  const blocking = violations.filter((v) => v.impact === "serious" || v.impact === "critical");

  if (opts.knownGap) {
    // ── KNOWN-GAP tier: only the beaded serious rule-ids are tolerated; NEW ones fail. ──────────
    // In SEED mode this surface's allowance is being RE-MEASURED — writeSeedBaseline captures the
    // currently-observed serious rule-ids as `allowedSeriousRules` and the author reviews the
    // seeded diff — so there is no fixed allowance to enforce against yet; skip while seeding.
    if (!SEED) {
      const allowed = new Set(baseline.surfaces[key]?.allowedSeriousRules ?? []);
      const regressions = blocking.filter((v) => !allowed.has(v.id));
      expect(
        regressions,
        `\n[a11y ${key}] NEW serious/critical WCAG 2.1 AA violation on a KNOWN-GAP surface ` +
          `(allowance: [${[...allowed].join(", ")}], tracked in ${opts.knownGap.bead}) — a regression ` +
          `beyond the beaded set:\n${formatViolations(regressions)}\n`,
      ).toHaveLength(0);
    }
  } else {
    // ── ZERO tier: unconditional hard fail on any serious/critical — ENFORCED EVEN IN SEED mode.
    // The zero bar is never read from the baseline, so this bites before/independent of the seed
    // early-return below: a re-seed can never be a hole that blesses a NEW serious/critical bug on
    // a supposedly-clean surface by writing it into the baseline. ────────────────────────────────
    expect(
      blocking,
      `\n[a11y ${key}] serious/critical WCAG 2.1 AA violation(s) — must be ZERO:\n${formatViolations(
        blocking,
      )}\n`,
    ).toHaveLength(0);
  }

  // In SEED mode the moderate/minor ceiling (and any known-gap allowance) is purely RE-MEASURED,
  // not enforced: writeSeedBaseline rewrites the ratchet from these counts in afterAll and the
  // author reviews the seeded baseline before committing. The unconditional zero-tier serious bar
  // above has ALREADY bitten, so returning here can never bless a new serious/critical violation.
  if (SEED) return counts;

  // ── RATCHET (both tiers): moderate + minor may only FALL vs the committed ceiling. ────────────
  const ceiling = baseline.surfaces[key];
  expect(
    ceiling,
    `[a11y ${key}] no baseline entry — re-seed with A11Y_SEED=1 (the ratchet needs a ceiling).`,
  ).toBeTruthy();

  const byImpact = (impact: Impact) => violations.filter((v) => (v.impact ?? "minor") === impact);
  expect(
    counts.moderate,
    `[a11y ${key}] moderate violations rose ${ceiling.moderate} → ${counts.moderate} — the ratchet ` +
      `may only fall. Fix the new issue, or re-seed (A11Y_SEED=1) after an intentional drop.\n` +
      `${formatViolations(byImpact("moderate"))}\n`,
  ).toBeLessThanOrEqual(ceiling.moderate);
  expect(
    counts.minor,
    `[a11y ${key}] minor violations rose ${ceiling.minor} → ${counts.minor} — the ratchet may only ` +
      `fall. Fix the new issue, or re-seed (A11Y_SEED=1) after an intentional drop.\n` +
      `${formatViolations(byImpact("minor"))}\n`,
  ).toBeLessThanOrEqual(ceiling.minor);

  return counts;
}

/** In A11Y_SEED mode, rewrite bench/a11y-baseline.json from the measured counts (call in afterAll). */
export function writeSeedBaseline(): void {
  if (!SEED || measured.size === 0) return;
  const surfaces: Record<string, SurfaceBaseline> = {};
  for (const key of [...measured.keys()].sort()) {
    const m = measured.get(key)!;
    surfaces[key] = m.knownGap
      ? {
          ...m.counts,
          allowedSeriousRules: m.seriousRules,
          bead: m.knownGap.bead,
          reason: m.knownGap.reason,
        }
      : { ...m.counts };
  }
  const doc = {
    _comment: BASELINE_COMMENT,
    wcagTags: [...WCAG_TAGS],
    ruleExclusions: baseline.ruleExclusions ?? {},
    surfaces,
  };
  writeFileSync(BASELINE_PATH, `${JSON.stringify(doc, null, 2)}\n`);
  // eslint-disable-next-line no-console
  console.log(`[a11y] re-seeded ${Object.keys(surfaces).length} states → ${BASELINE_PATH}`);
}
