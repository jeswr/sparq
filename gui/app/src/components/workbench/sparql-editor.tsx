"use client";

// [OPUS-4.8] sq-ixc3.12 — the workbench SPARQL editor, the operational sibling of the site's
// `site/src/components/sparql-editor.tsx` (sq-n5aw). Same dependency-free overlay technique: a
// highlighted `<pre>` rendered behind a transparent-text `<textarea>`, both sharing identical
// font metrics / padding / scroll so glyphs align exactly. The tokenizer + the missing-prefix
// helper are the framework-agnostic `@sparq/client` core (`tokenizeSparql` /
// `missingCommonPrefixes` / `withPrefixes`), so the SAME highlighting serves the site and this
// Tauri webview — the design's load-bearing "part (A) transfers unchanged" property.
//
// It is DEPENDENCY-FREE (no CodeMirror / Monaco): no new npm dependency, no bundle-size hit,
// and static-export-safe. The workbench differs from the site editor only in chrome: it is
// FULL-HEIGHT (the query pane drives the height) rather than a fixed `rows` document, so the
// editing surface fills the workbench's top pane.

import * as React from "react";
import { Sparkles } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { handleEditorKeyDown } from "@/lib/editor-keys";
import {
  type SparqlToken,
  tokenizeSparql,
  missingCommonPrefixes,
  withPrefixes,
} from "@sparq/client";

// The shared text-metrics: the highlight `<pre>` and the `<textarea>` MUST use identical font,
// size, line-height, padding, tab size and wrapping, or the layers drift. Centralised here so
// they can never diverge.
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
  /** ⌘/Ctrl-Enter (and any other) keydown handler on the editing textarea. */
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  /** Accessible label for the editing surface. */
  ariaLabel?: string;
  /** id for the textarea (so the e2e harness + an external `<label htmlFor>` can target it). */
  id?: string;
  className?: string;
}

/**
 * A FULL-HEIGHT syntax-highlighting SPARQL editor: a transparent-caret `<textarea>` over a
 * highlighted `<pre>`. Fully controlled; emits edits (and prefix insertions) through
 * {@link SparqlEditorProps.onChange}. Fills the height of its (flex) parent.
 */
export function WorkbenchSparqlEditor({
  value,
  onChange,
  onKeyDown,
  ariaLabel = "SPARQL query editor",
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

  // (sq-rcuvq) Merge editor hotkeys (Ctrl+/, Tab, Shift+Tab) with any parent-provided handler.
  // The hotkeys handler calls e.preventDefault() for its keys; the parent handler (e.g.
  // Ctrl+Enter → run query) is always called after so its own keys still fire.
  const mergedKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      handleEditorKeyDown(e, "#", onChange);
      onKeyDown?.(e);
    },
    [onChange, onKeyDown],
  );

  const tokens = React.useMemo(() => tokenizeSparql(value), [value]);

  // The well-known prefixes the query USES but does not DECLARE — the one-click "add prefixes"
  // affordance. Empty (hidden) when the query already declares everything it uses.
  const missing = React.useMemo(() => missingCommonPrefixes(value), [value]);

  const addMissingPrefixes = React.useCallback(() => {
    if (missing.length === 0) return;
    onChange(withPrefixes(value, missing));
  }, [missing, onChange, value]);

  return (
    <div className={cn("flex min-h-0 flex-1 flex-col", className)}>
      <div className="relative min-h-0 flex-1 bg-background">
        {/* Highlight layer: aria-hidden (the textarea is the accessible control). A trailing
            newline keeps the last line's height stable while typing at the very end. */}
        <pre
          ref={preRef}
          aria-hidden="true"
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

        {/* Editing layer: transparent text (so the highlight shows through) but a visible caret.
            `spellCheck` off — SPARQL is not prose. The id stays `repl-query` so the existing
            tauri-driver e2e (gui/e2e/run-e2e.mjs) keeps finding the editor. */}
        <textarea
          id={id}
          aria-label={ariaLabel}
          value={value}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={mergedKeyDown}
          onScroll={(e) => syncScroll(e.currentTarget)}
          className={cn(
            "absolute inset-0 block h-full w-full resize-none bg-transparent text-transparent caret-foreground outline-none",
            "selection:bg-primary/30",
            EDITOR_PAD_CLASS,
            EDITOR_TEXT_CLASS,
          )}
        />
      </div>

      {missing.length > 0 && (
        <div className="flex items-center gap-2 border-t bg-card px-3 py-1 text-xs text-muted-foreground">
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
            className="ml-auto h-6 gap-1 px-2 text-xs"
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
