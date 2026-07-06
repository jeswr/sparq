"use client";

// [OPUS-4.8] sq-n5aw — query-editor uplift (MVP). A real SPARQL editor surface that replaces
// the plain `<textarea>` in the live REPL (`research/gui-design.md` §2, MVP item 1 — the
// single biggest real UX gap). It gives the playground:
//   * SPARQL SYNTAX HIGHLIGHTING — a highlighted `<pre>` rendered behind a transparent-text
//     `<textarea>` (the standard zero-dependency overlay technique). The two layers share
//     identical font metrics, padding and scroll offset so glyphs align exactly. The
//     tokenizer is the framework-agnostic `@sparq/client` core (`tokenizeSparql`), so the SAME
//     highlighting serves the site and the Tauri webview.
//   * COMMON-PREFIX awareness — when the query uses a well-known prefix it has not declared
//     (`foaf:`, `rdf:`, …), a one-click affordance prepends the missing `PREFIX` lines
//     (`missingCommonPrefixes` / `withPrefixes`, also from `@sparq/client`).
//
// This is DEPENDENCY-FREE (no CodeMirror / Monaco): no new npm dependency, no bundle-size
// hit, and static-export-safe (no SSR-incompatible client lib). The trade-off vs a full
// editor framework is no autocomplete popovers / multi-cursor; for the MVP, highlighting +
// prefix help is the high-leverage win, and the surface can grow later. Example-query
// chips stay owned by the REPL (they pick a whole query); this component owns the EDITING.

import * as React from "react";
import { Sparkles } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  type SparqlToken,
  tokenizeSparql,
  missingCommonPrefixes,
  withPrefixes,
} from "@sparq/client";

// The shared text-metrics: the highlight `<pre>` and the `<textarea>` MUST use identical
// font, size, line-height, padding, tab size and wrapping, or the layers drift. Centralised
// here so they can never diverge.
const EDITOR_TEXT_CLASS =
  "font-mono text-[13px] leading-relaxed whitespace-pre-wrap break-words [tab-size:2]";
const EDITOR_PAD_CLASS = "p-3";

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

export interface SparqlEditorProps {
  /** Current query text (controlled). */
  value: string;
  /** Called with the new text on edit (and when a prefix block is inserted). */
  onChange: (next: string) => void;
  /** Visible rows (the textarea is resizable vertically from here). */
  rows?: number;
  /** Accessible label for the editing surface. */
  ariaLabel?: string;
  /** id for the textarea (so an external `<label htmlFor>` can target it). */
  id?: string;
  className?: string;
}

/**
 * A syntax-highlighting SPARQL editor: a transparent-caret `<textarea>` over a highlighted
 * `<pre>`. Fully controlled; emits edits (and prefix insertions) through {@link onChange}.
 */
export function SparqlEditor({
  value,
  onChange,
  rows = 9,
  ariaLabel = "SPARQL query",
  id,
  className,
}: SparqlEditorProps) {
  const preRef = React.useRef<HTMLPreElement>(null);

  // Keep the highlight layer's scroll offset in lockstep with the textarea (long queries that
  // overflow vertically/horizontally must scroll both layers together).
  const syncScroll = React.useCallback((el: HTMLTextAreaElement) => {
    if (preRef.current) {
      preRef.current.scrollTop = el.scrollTop;
      preRef.current.scrollLeft = el.scrollLeft;
    }
  }, []);

  const tokens = React.useMemo(() => tokenizeSparql(value), [value]);

  // The well-known prefixes the query USES but does not DECLARE — the one-click "add prefixes"
  // affordance. Empty (hidden) when the query already declares everything it uses.
  const missing = React.useMemo(() => missingCommonPrefixes(value), [value]);

  const addMissingPrefixes = React.useCallback(() => {
    if (missing.length === 0) return;
    onChange(withPrefixes(value, missing));
  }, [missing, onChange, value]);

  return (
    <div className={cn("space-y-2", className)}>
      <div className="relative rounded-lg border bg-muted/40">
        {/* Highlight layer: aria-hidden (the textarea is the accessible control). A trailing
            newline keeps the last line's height stable while typing at the very end.
            [FABLE-5] sq-ymr2e.9 — tabIndex -1: modern Chromium makes scrollable containers
            with no focusable children KEYBOARD-focusable, which put this aria-hidden layer in
            the Tab order (focus on aria-hidden content — a WCAG 4.1.2-class bug axe cannot
            see; caught by e2e/a11y-keyboard.spec.ts). Scrolling stays available through the
            textarea, whose scroll this layer mirrors. */}
        <pre
          ref={preRef}
          aria-hidden="true"
          tabIndex={-1}
          className={cn(
            "pointer-events-none absolute inset-0 m-0 overflow-auto",
            EDITOR_PAD_CLASS,
            EDITOR_TEXT_CLASS,
          )}
        >
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
          {"\n"}
        </pre>

        {/* Editing layer: transparent text (so the highlight shows through) but a visible
            caret. `spellCheck` off — SPARQL is not prose. */}
        <textarea
          id={id}
          aria-label={ariaLabel}
          value={value}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          rows={rows}
          onChange={(e) => onChange(e.target.value)}
          onScroll={(e) => syncScroll(e.currentTarget)}
          className={cn(
            "relative block w-full resize-y bg-transparent text-transparent caret-foreground outline-none",
            "selection:bg-primary/30 focus-visible:ring-3 focus-visible:ring-ring/40 rounded-lg",
            EDITOR_PAD_CLASS,
            EDITOR_TEXT_CLASS,
          )}
        />
      </div>

      {missing.length > 0 && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>
            Uses{" "}
            <span className="font-mono text-foreground">
              {missing.map((b) => `${b.prefix}:`).join(" ")}
            </span>{" "}
            without a declaration.
          </span>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-6 gap-1 px-2 text-xs"
            onClick={addMissingPrefixes}
          >
            <Sparkles className="size-3" />
            Add {missing.length === 1 ? "prefix" : `${missing.length} prefixes`}
          </Button>
        </div>
      )}
    </div>
  );
}
