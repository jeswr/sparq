"use client";

// [OPUS-4.8] sq-vw3ax — read-only SPARQL syntax-highlight renderer for the /capabilities hero
// code panel. The display counterpart of RdfHighlight (which does Turtle): it tokenizes the
// query with the dependency-free `@sparq/client` `tokenizeSparql` core and wraps each token in
// the SAME `.sq-tok-*` classes globals.css already defines, so highlighting follows light/dark
// automatically and adds NO new palette and NO new dependency. Pure presentation, static-export
// safe — the gap-free tokenizer reproduces the source query exactly.

import * as React from "react";

import { cn } from "@/lib/utils";
import { type SparqlToken, tokenizeSparql } from "@sparq/client";

const TOKEN_CLASS: Record<SparqlToken["type"], string> = {
  keyword: "sq-tok-keyword",
  variable: "sq-tok-variable",
  iri: "sq-tok-iri",
  prefixed: "sq-tok-prefixed",
  string: "sq-tok-string",
  number: "sq-tok-number",
  comment: "sq-tok-comment",
  punctuation: "",
  plain: "",
};

export function SparqlHighlight({
  query,
  className,
}: {
  query: string;
  className?: string;
}) {
  const tokens = React.useMemo(() => tokenizeSparql(query), [query]);
  return (
    <pre className={cn("font-mono", className)} aria-label="Example SPARQL query">
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
