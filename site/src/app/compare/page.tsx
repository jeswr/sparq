// [OPUS-4.8] sq-vw3ax.16 — /compare: the honest competitive feature matrix.
//
// Every vendor in this space publishes a comparison page; sparq's differentiator is
// radical honesty. So this page KEEPS the rows where sparq is behind (GAP) or only
// partly there (PARTIAL), labels the research-grade ZK/MPC estate as unaudited, and
// — critically — carries ZERO performance numbers. Comparative, same-box measured
// evidence is itself open work (beads sq-hmd7l / sq-vw3ax.12); this page LINKS the
// benchmark dashboard and never inlines a figure.
//
// The page is a pure renderer over data/competitive-matrix.ts (a faithful
// transcription of research/competitive-feature-analysis-2026-07.md §1). A PARTIAL/GAP
// row that names TRACKED future work links to its governing bead (build-in-public),
// resolved via a GitHub issues search on the bead id — issues carry a `bd-id:` marker,
// so this stays correct without a hand-maintained bead→issue-number map. A few
// explicitly-deferred items are not yet beaded and say so in the note; the invariant is
// enforced by test/competitive-matrix.test.mjs.
//
// Nav: Cmd-K only (a TOP_PAGES entry in command-palette.tsx), like /assurance,
// /dogfooding and /genai-retrieval — the slim top bar stays at its fixed size.

import type { Metadata } from "next";
import Link from "next/link";
import { ArrowUpRight, GitCompareArrows, Scale } from "lucide-react";

import {
  COMPETITORS,
  MATRIX,
  type Mark,
  type MatrixRow,
  type Tier,
} from "@/data/competitive-matrix";

const REPO = "https://github.com/sparq-org/sparq";
const DESIGN_RECORD = `${REPO}/blob/main/research/competitive-feature-analysis-2026-07.md`;

export const metadata: Metadata = {
  title: "How sparq compares — honest feature matrix",
  description:
    "sparq versus RDFox, Stardog, GraphDB, TopBraid and the enterprise cluster — an honest, build-in-public feature matrix that keeps the GAP rows visible, labels the unaudited research estate, and carries zero performance numbers (measured comparisons live on the benchmarks dashboard).",
};

/** External link to a repo/tracker artifact, with the small up-right glyph. */
function Artifact({ href, label }: { href: string; label: string }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 text-primary hover:underline"
    >
      {label}
      <ArrowUpRight className="size-3.5 opacity-70" aria-hidden />
    </a>
  );
}

// ── Tier badge ────────────────────────────────────────────────────────────────
// Five honest buckets. GAP and PARTIAL are shown as prominently as LEAD — that is
// the whole point of the page.
const TIER_STYLE: Record<Tier, string> = {
  LEAD: "bg-[color-mix(in_oklch,var(--success)_15%,transparent)] text-[var(--success-on-tint)]",
  PARITY: "bg-primary/10 text-primary",
  PARTIAL:
    "bg-[color-mix(in_oklch,var(--warning)_18%,transparent)] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]",
  GAP: "bg-[color-mix(in_oklch,var(--destructive)_14%,transparent)] text-[color-mix(in_oklch,var(--destructive)_82%,var(--foreground))]",
  SKIP: "bg-muted text-muted-foreground",
};

function TierBadge({ tier, label }: { tier: Tier; label?: string }) {
  return (
    <span
      className={`inline-flex h-5 w-fit items-center justify-center whitespace-nowrap rounded-4xl px-2 text-xs font-semibold ${TIER_STYLE[tier]}`}
    >
      {label ?? tier}
    </span>
  );
}

// ── Competitor mark ─────────────────────────────────────────────────────────
const MARK_GLYPH: Record<Mark, string> = { yes: "✓", partial: "~", no: "—" };
const MARK_LABEL: Record<Mark, string> = {
  yes: "has it",
  partial: "partial or dated",
  no: "lacks it",
};
const MARK_CLASS: Record<Mark, string> = {
  yes: "text-[var(--success-on-tint)]",
  partial: "text-muted-foreground",
  no: "text-muted-foreground/45",
};

function MarkCell({ mark }: { mark: Mark }) {
  return (
    <td className="px-3 py-2.5 text-center">
      <span className={`font-semibold ${MARK_CLASS[mark]}`} title={MARK_LABEL[mark]}>
        {MARK_GLYPH[mark]}
        <span className="sr-only">{MARK_LABEL[mark]}</span>
      </span>
    </td>
  );
}

/** Build-in-public: link a bead id to its tracker issue via an id search. */
function beadHref(bead: string) {
  return `${REPO}/issues?q=${encodeURIComponent(`is:issue ${bead}`)}`;
}

function BeadLinks({ beads }: { beads: string[] }) {
  return (
    <span className="mt-1.5 flex flex-wrap gap-1.5">
      {beads.map((b) => (
        <a
          key={b}
          href={beadHref(b)}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1 rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground hover:text-primary"
          title={`Track ${b} on the issue tracker`}
        >
          {b}
          <ArrowUpRight className="size-3 opacity-60" aria-hidden />
        </a>
      ))}
    </span>
  );
}

function Row({ row }: { row: MatrixRow }) {
  return (
    <tr className="border-b align-top last:border-0 hover:bg-muted/30">
      <th scope="row" className="px-3 py-2.5 text-left font-medium text-foreground">
        {row.feature}
      </th>
      <MarkCell mark={row.rdfox} />
      <MarkCell mark={row.stardog} />
      <MarkCell mark={row.graphdb} />
      <MarkCell mark={row.topbraid} />
      <MarkCell mark={row.ent} />
      <td className="px-3 py-2.5 text-muted-foreground">
        <TierBadge tier={row.tier} label={row.tierLabel} />
        <p className="mt-1.5 leading-6">{row.note}</p>
        {row.beads && row.beads.length > 0 ? <BeadLinks beads={row.beads} /> : null}
      </td>
    </tr>
  );
}

const LEGEND_TIERS: { tier: Tier; blurb: string }[] = [
  { tier: "LEAD", blurb: "sparq is ahead of the named competitors on breadth, architecture, or (where measured) perf." },
  { tier: "PARITY", blurb: "a comparable capability exists." },
  { tier: "PARTIAL", blurb: "real machinery exists, but a specific competitor capability is missing or unmeasured — the delta is named." },
  { tier: "GAP", blurb: "sparq does not have it yet; the worth-building verdict and bead are given." },
  { tier: "SKIP", blurb: "a deliberate non-goal, with the reason — not a gap we intend to fill." },
];

export default function ComparePage() {
  return (
    <div className="space-y-10">
      {/* ── Header ─────────────────────────────────────────────── */}
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <GitCompareArrows className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">How sparq compares</h1>
            <p className="text-sm text-muted-foreground">
              An honest feature matrix — the GAP rows stay visible.
            </p>
          </div>
        </div>
      </header>

      {/* ── Lead ───────────────────────────────────────────────── */}
      <section className="measure space-y-3 text-sm text-muted-foreground">
        <p>
          Every engine vendor publishes a comparison page, and every one of them shows
          only where it wins. This one is different on purpose: it keeps the rows where
          sparq is <strong className="text-foreground">behind</strong> (GAP) or only
          part of the way there (PARTIAL), it labels the research-grade privacy estate
          as unaudited, and it carries{" "}
          <strong className="text-foreground">no performance numbers at all</strong>.
          Measured, same-box comparative evidence is itself open work — it lives on the{" "}
          <Link href="/benchmarks" className="text-primary underline underline-offset-2">
            benchmarks dashboard
          </Link>
          , never inlined here.
        </p>
        <p>
          The matrix is a faithful transcription of the internal five-way competitor
          analysis (
          <Artifact href={DESIGN_RECORD} label="the design record" />
          ). Competitor marks are their <em>published</em> capabilities as surveyed
          there — <span className="whitespace-nowrap">✓ = has it</span>,{" "}
          <span className="whitespace-nowrap">~ = partial or dated</span>,{" "}
          <span className="whitespace-nowrap">— = lacks it</span>. Where a PARTIAL/GAP
          row names <em>tracked</em> future work it links to its governing bead, so this
          page is a build-in-public roadmap as much as a comparison; a few
          explicitly-deferred items are not yet beaded and say so.
        </p>
      </section>

      {/* ── Legend ─────────────────────────────────────────────── */}
      <section aria-labelledby="legend-title" className="space-y-4">
        <div className="flex items-center gap-2">
          <Scale className="size-4 text-primary" aria-hidden />
          <h2 id="legend-title" className="text-lg font-semibold">
            How to read the sparq column
          </h2>
        </div>
        <dl className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {LEGEND_TIERS.map(({ tier, blurb }) => (
            <div key={tier} className="rounded-lg border bg-card px-3 py-2.5">
              <dt>
                <TierBadge tier={tier} />
              </dt>
              <dd className="mt-1.5 text-sm text-muted-foreground">{blurb}</dd>
            </div>
          ))}
        </dl>
        <p className="text-xs text-muted-foreground">
          &ldquo;Ent.&rdquo; is the enterprise cluster surveyed together — Neptune,
          AllegroGraph, Virtuoso, Neo4j-n10s and MarkLogic; a mark means at least one of
          them has the feature.
        </p>
      </section>

      {/* ── Section tables ─────────────────────────────────────── */}
      {MATRIX.map((section) => (
        <section key={section.id} id={section.id} className="space-y-3 scroll-mt-24">
          <h2 className="text-lg font-semibold">{section.title}</h2>
          <div className="overflow-x-auto rounded-xl border">
            <table className="w-full border-collapse text-left text-sm">
              <thead>
                <tr className="border-b bg-muted/40 text-xs uppercase tracking-wide text-muted-foreground">
                  <th scope="col" className="px-3 py-2.5 font-medium">
                    Feature
                  </th>
                  {COMPETITORS.map((c) => (
                    <th key={c.key} scope="col" className="px-3 py-2.5 text-center font-medium">
                      {c.label}
                    </th>
                  ))}
                  <th scope="col" className="px-3 py-2.5 font-medium">
                    sparq — honest standing
                  </th>
                </tr>
              </thead>
              <tbody>
                {section.rows.map((row) => (
                  <Row key={row.feature} row={row} />
                ))}
              </tbody>
            </table>
          </div>
        </section>
      ))}

      {/* ── What this page does NOT claim ──────────────────────── */}
      <section aria-labelledby="notclaim-title" className="space-y-4">
        <h2 id="notclaim-title" className="text-lg font-semibold">
          What this page does not claim
        </h2>
        <ul className="space-y-3 text-sm text-muted-foreground">
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">No performance claims.</strong> There is
            not a single benchmark figure on this page, by design. Comparative,
            same-box measured results are open work (beads{" "}
            <Artifact href={beadHref("sq-hmd7l")} label="sq-hmd7l" /> and{" "}
            <Artifact href={beadHref("sq-vw3ax.12")} label="sq-vw3ax.12" />); until they
            land, the honest answer is &ldquo;unmeasured&rdquo;, and any real numbers
            appear only on the{" "}
            <Link href="/benchmarks" className="text-primary underline underline-offset-2">
              benchmarks dashboard
            </Link>
            .
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">
              Competitor marks are their published capabilities, not our measurements.
            </strong>{" "}
            They are surveyed from vendor documentation and reflect what each product
            claims; they are not head-to-head test results.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">
              The ZK / MPC estate is research-grade and unaudited.
            </strong>{" "}
            sparq is the only entry that offers verifiable-query / zero-knowledge / MPC
            work at all, but the v1 verifier is pending external cryptographer sign-off
            (<Artifact href={beadHref("sq-qhy4")} label="sq-qhy4" />) and sparq-mpc is
            honest-majority semi-honest only. No settled privacy or soundness guarantee
            is claimed here.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">
              &ldquo;LEAD&rdquo; is not &ldquo;best at everything&rdquo;.
            </strong>{" "}
            A LEAD row means sparq is ahead on that specific axis (usually breadth or
            architecture). Where a purpose-built system is more polished — raw ANN
            perf, for one — the row says so.
          </li>
        </ul>
      </section>

      {/* ── Front doors / CTAs ─────────────────────────────────── */}
      <section className="flex flex-wrap gap-3 border-t pt-6 text-sm">
        <Link
          href="/capabilities"
          className="inline-flex items-center rounded-lg border px-3 py-2 font-medium hover:bg-muted/50"
        >
          Explore every capability
        </Link>
        <Link
          href="/capabilities/shacl-forms"
          className="inline-flex items-center rounded-lg border px-3 py-2 font-medium hover:bg-muted/50"
        >
          SHACL-driven forms — the white-space bet
        </Link>
        <Link
          href="/benchmarks"
          className="inline-flex items-center rounded-lg border px-3 py-2 font-medium hover:bg-muted/50"
        >
          Benchmarks dashboard
        </Link>
        <a
          href={DESIGN_RECORD}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1 rounded-lg border px-3 py-2 font-medium hover:bg-muted/50"
        >
          Read the full analysis
          <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
        </a>
      </section>
    </div>
  );
}
