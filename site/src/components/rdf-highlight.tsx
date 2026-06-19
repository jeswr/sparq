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
import { type TurtleToken, tokenizeTurtle } from "@sparq/client";

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
