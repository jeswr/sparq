"use client";

// [OPUS-4.8] sq-ixc3.1 — a syntax-highlighting EDITOR for JSON-LD input.
//
// The JSON-LD counterpart of `RdfEditor` (Turtle, sq-8uew) and `SparqlEditor` (sq-n5aw): a
// transparent-text `<textarea>` rendered over a highlighted `<pre>`, the standard
// zero-dependency overlay technique. The two layers share identical font metrics, padding, tab
// size and scroll offset so glyphs align exactly. Tokenization is the framework-agnostic
// `@sparq/client` `tokenizeJsonLd` core, and the spans reuse the SAME `.sq-tok-*` classes
// `globals.css` already defines for the SPARQL/Turtle editors — no new palette, theme-following
// highlighting, no new npm dependency, static-export safe. Used in the data-formats parse panel
// when the JSON-LD format is picked. The lighter read-only display counterpart is
// `JsonLdHighlight`.

import * as React from "react";

import { cn } from "@/lib/utils";
import { type JsonLdToken, tokenizeJsonLd } from "@sparq/client";

// Shared text-metrics: the highlight `<pre>` and the `<textarea>` MUST use identical font,
// size, line-height, padding, tab size and wrapping, or the layers drift. Matches `RdfEditor`.
const EDITOR_TEXT_CLASS =
  "font-mono text-[13px] leading-relaxed whitespace-pre-wrap break-words [tab-size:2]";
const EDITOR_PAD_CLASS = "p-3";

// JSON-LD uses keyword (`@…` / true/false/null), prefixed (object keys), string (values),
// number and punctuation. `iri` / `variable` / `comment` never occur in JSON, but the shared
// token-type map keeps every `SparqlTokenType` key so one renderer styles all three languages.
const TOKEN_CLASS: Record<JsonLdToken["type"], string> = {
  keyword: "sq-tok-keyword",
  variable: "", // unused by JSON-LD
  iri: "", // unused by JSON-LD
  prefixed: "sq-tok-prefixed", // object keys
  string: "sq-tok-string",
  number: "sq-tok-number",
  comment: "", // JSON has no comments
  punctuation: "",
  plain: "",
};

export interface JsonLdEditorProps {
  /** Current document text (controlled). */
  value: string;
  /** Called with the new text on edit. */
  onChange: (next: string) => void;
  /** Visible rows (the textarea is resizable vertically from here). */
  rows?: number;
  /** Accessible label for the editing surface. */
  ariaLabel?: string;
  /** id for the textarea (so an external `<label htmlFor>` can target it). */
  id?: string;
  /** Disable editing (e.g. while the engine is warming). */
  disabled?: boolean;
  className?: string;
}

/**
 * A syntax-highlighting JSON-LD editor: a transparent-caret `<textarea>` over a highlighted
 * `<pre>`. Fully controlled; emits edits through {@link onChange}.
 */
export function JsonLdEditor({
  value,
  onChange,
  rows = 10,
  ariaLabel = "JSON-LD document",
  id,
  disabled,
  className,
}: JsonLdEditorProps) {
  const preRef = React.useRef<HTMLPreElement>(null);

  // Keep the highlight layer's scroll offset in lockstep with the textarea.
  const syncScroll = React.useCallback((el: HTMLTextAreaElement) => {
    if (preRef.current) {
      preRef.current.scrollTop = el.scrollTop;
      preRef.current.scrollLeft = el.scrollLeft;
    }
  }, []);

  const tokens = React.useMemo(() => tokenizeJsonLd(value), [value]);

  return (
    <div className={cn("relative rounded-lg border bg-muted/40", className)}>
      {/* Highlight layer: aria-hidden (the textarea is the accessible control). A trailing
          newline keeps the last line's height stable while typing at the very end.
          [FABLE-5] sq-ymr2e.9 — tabIndex -1: keeps this aria-hidden scroller OUT of the
          keyboard Tab order (Chromium keyboard-focusable scrollers; see sparql-editor.tsx). */}
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
          caret. `spellCheck` off — JSON-LD is not prose. */}
      <textarea
        id={id}
        aria-label={ariaLabel}
        value={value}
        spellCheck={false}
        autoCapitalize="off"
        autoCorrect="off"
        rows={rows}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        onScroll={(e) => syncScroll(e.currentTarget)}
        className={cn(
          "relative block w-full resize-y bg-transparent text-transparent caret-foreground outline-none",
          "selection:bg-primary/30 focus-visible:ring-3 focus-visible:ring-ring/40 rounded-lg",
          "disabled:cursor-not-allowed disabled:opacity-60",
          EDITOR_PAD_CLASS,
          EDITOR_TEXT_CLASS,
        )}
      />
    </div>
  );
}

/**
 * Read-only JSON-LD syntax-highlight renderer — the display counterpart of {@link JsonLdEditor}
 * (mirrors `RdfHighlight` for Turtle). Renders a plain styled `<pre>`, so it drops in anywhere a
 * `<pre>` already shows JSON-LD text. The tokenizer is gap-free, so the rendered text reproduces
 * `text` exactly.
 */
export function JsonLdHighlight({
  text,
  className,
  ...rest
}: {
  /** The JSON-LD document to highlight. */
  text: string;
  /** Classes for the `<pre>` host (sizing, scroll, border, padding, font). */
  className?: string;
  /** Optional `data-*` hook for tests/inspection. */
  "data-testid"?: string;
}) {
  const tokens = React.useMemo(() => tokenizeJsonLd(text), [text]);
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
