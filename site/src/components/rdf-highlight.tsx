// [OPUS-4.8] sq-8uew — a read-only Turtle/TriG/N-Triples/N-Quads syntax-highlight renderer.
//
// Where Turtle-family RDF is DISPLAYED (CONSTRUCT/DESCRIBE graph output, the dataset N-Triples
// view, the data-formats sample read-back) we want the same theme-aware colours the SPARQL
// editor already shows — keywords, IRIs, prefixed names, strings, numbers, comments — instead
// of a flat monochrome `<pre>`. This component is the display counterpart of `SparqlEditor`:
// it tokenizes via the dependency-free `@sparq/client` `tokenizeTurtle` core and wraps each
// token in a `.sq-tok-*` span (the SAME classes `globals.css` already defines for SPARQL, so
// there is no new palette and highlighting follows light/dark automatically).
//
// It renders a plain styled `<pre>` (NOT an editor overlay) so it drops in anywhere a `<pre>`
// already shows RDF text. Styling props (`className`) are forwarded so each call site keeps
// its existing sizing/scroll/border. Dependency-free, static-export-safe.

import * as React from "react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  type PrefixBinding,
  type TurtleToken,
  declaredPrefixBindings,
  prettyTurtle,
  tokenizeTurtle,
} from "@sparq/client";

const TOKEN_CLASS: Record<TurtleToken["type"], string> = {
  keyword: "sq-tok-keyword",
  variable: "sq-tok-variable", // unused by Turtle, kept for the shared token-type map
  iri: "sq-tok-iri",
  prefixed: "sq-tok-prefixed",
  string: "sq-tok-string",
  number: "sq-tok-number",
  comment: "sq-tok-comment",
  punctuation: "",
  plain: "",
};

export interface RdfHighlightProps {
  /** The RDF document (Turtle / TriG / N-Triples / N-Quads) to highlight. */
  text: string;
  /** Classes for the `<pre>` host (sizing, scroll, border, padding, font). */
  className?: string;
  /** Optional `data-*` hook for tests/inspection. */
  "data-testid"?: string;
}

/**
 * Render an RDF document as a syntax-highlighted, read-only `<pre>`. Pure presentation: the
 * tokenizer is gap-free, so the rendered text reproduces `text` exactly.
 */
export function RdfHighlight({ text, className, ...rest }: RdfHighlightProps) {
  const tokens = React.useMemo(() => tokenizeTurtle(text), [text]);
  return (
    <pre className={cn("font-mono", className)} {...rest}>
      {tokens.map((t, idx) => {
        const cls = TOKEN_CLASS[t.type];
        return cls ? (
          <span key={idx} className={cls}>
            {t.text}
          </span>
        ) : (
          <React.Fragment key={idx}>{t.text}</React.Fragment>
        );
      })}
    </pre>
  );
}

// [OPUS-4.8] sq-gb4o (#805) — a graph-result view that reshapes the engine's FLAT N-Triples/
// N-Quads into idiomatic, grouped, indented Turtle (`@prefix` abbreviation, subject blocks,
// `;`/`,` lists, RDF 1.2 `<<( s p o )>>` triple terms) before highlighting it. "Pretty" is the
// default; a toggle shows the engine's raw N-Triples/N-Quads (round-trip-equivalent — the same
// triples either way). The pretty pass is best-effort and NEVER throws (a line it cannot parse
// is passed through verbatim), but we still wrap it defensively so a formatting failure falls
// back to raw rather than blanking the result.

export interface TurtleResultProps {
  /** The RDF document the engine emitted (N-Triples for CONSTRUCT/DESCRIBE, N-Quads for a
   * named-graph dataset view). */
  text: string;
  /** The SPARQL the graph was produced from, if any — its `PREFIX` declarations are reused to
   * abbreviate result IRIs (the user's own prefixes win over the well-known registry). */
  query?: string;
  /** Classes for the highlighted `<pre>` host (sizing, scroll, border, padding, font). */
  className?: string;
  /** The label shown on the raw toggle — "N-Triples" for a triple graph, "N-Quads" for a
   * dataset with named graphs. Defaults to "N-Triples". */
  rawLabel?: string;
  /** Optional `data-*` hook for tests/inspection (applied to the `<pre>`). */
  "data-testid"?: string;
}

/** Pretty/raw Turtle result view: pretty-print THEN highlight, with a view toggle. */
export function TurtleResult({
  text,
  query,
  className,
  rawLabel = "N-Triples",
  ...rest
}: TurtleResultProps) {
  const [view, setView] = React.useState<"pretty" | "raw">("pretty");

  const pretty = React.useMemo(() => {
    try {
      const prefixes: PrefixBinding[] = query ? declaredPrefixBindings(query) : [];
      return prettyTurtle(text, { prefixes });
    } catch {
      // prettyTurtle is forgiving and should never throw, but never let a formatting bug blank
      // the result — fall back to the raw text.
      return text;
    }
  }, [text, query]);

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-1.5">
        <Button
          variant={view === "pretty" ? "default" : "outline"}
          size="sm"
          onClick={() => setView("pretty")}
          aria-pressed={view === "pretty"}
        >
          Pretty Turtle
        </Button>
        <Button
          variant={view === "raw" ? "default" : "outline"}
          size="sm"
          onClick={() => setView("raw")}
          aria-pressed={view === "raw"}
        >
          {rawLabel}
        </Button>
      </div>
      <RdfHighlight
        text={view === "pretty" ? pretty : text}
        className={className}
        data-turtle-view={view}
        {...rest}
      />
    </div>
  );
}
