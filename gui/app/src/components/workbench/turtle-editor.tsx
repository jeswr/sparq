"use client";

// [HAIKU-4.5] sq-5sjub — the workbench Turtle/SHACL/N3 editor, the operational sibling of
// `sparql-editor.tsx` (sq-ixc3.12). Same dependency-free overlay technique: a highlighted
// `<pre>` rendered behind a transparent-text `<textarea>`, both sharing identical font metrics
// / padding / scroll so glyphs align exactly. The tokenizer is the framework-agnostic
// `@sparq/client` core (`tokenizeTurtle`), which emits the same token classes as
// `tokenizeSparql`, so the EXISTING `.sq-tok-*` CSS palette applies unchanged.
//
// SHACL shapes are Turtle; N3 rules are Turtle with a superset of syntax (forward/backward
// rules, builtins, formula terms). The tokenizer treats N3 as Turtle (an acceptable
// approximation documented here) — no new tokenizer work.
//
// It is DEPENDENCY-FREE (no CodeMirror / Monaco): no new npm dependency, no bundle-size hit,
// and static-export-safe. The workbench differs from the site editor only in chrome: it is
// FULL-HEIGHT (the panel drives the height) rather than a fixed `rows` document, so the
// editing surface fills its parent container.

import * as React from "react";

import { cn } from "@/lib/utils";
import { handleEditorKeyDown } from "@/lib/editor-keys";
import { type SparqlToken, tokenizeTurtle } from "@sparq/client";

// The shared text-metrics: the highlight `<pre>` and the `<textarea>` MUST use identical font,
// size, line-height, padding, tab size and wrapping, or the layers drift. Centralised here so
// they can never diverge. **INVARIANT**: identical to the EDITOR_TEXT_CLASS and EDITOR_PAD_CLASS
// in sparql-editor.tsx so all editors align with the same metrics.
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

export interface TurtleEditorProps {
  /** Current Turtle/N3 text (controlled). */
  value: string;
  /** Called with the new text on edit. */
  onChange: (next: string) => void;
  /** ⌘/Ctrl-Enter (and any other) keydown handler on the editing textarea. */
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  /** Accessible label for the editing surface. */
  ariaLabel?: string;
  /** id for the textarea (so the e2e harness + an external `<label htmlFor>` can target it). */
  id?: string;
  /** Placeholder text shown when the textarea is empty. */
  placeholder?: string;
  className?: string;
}

/**
 * A FULL-HEIGHT syntax-highlighting Turtle/SHACL/N3 editor: a transparent-caret `<textarea>`
 * over a highlighted `<pre>`. Fully controlled; emits edits through
 * {@link TurtleEditorProps.onChange}. Fills the height of its (flex) parent.
 *
 * SHACL shapes are Turtle; N3 rules use Turtle with a superset of syntax (forward/backward
 * rules, builtins, formula terms). The tokenizer treats N3 as Turtle — an acceptable
 * approximation since highlighting is purely lexical.
 */
export function WorkbenchTurtleEditor({
  value,
  onChange,
  onKeyDown,
  ariaLabel = "Turtle/SHACL/N3 editor",
  id,
  placeholder,
  className,
}: TurtleEditorProps) {
  const preRef = React.useRef<HTMLPreElement>(null);

  // Keep the highlight layer's scroll offset in lockstep with the textarea (long documents that
  // overflow vertically/horizontally must scroll both layers together).
  const syncScroll = React.useCallback((el: HTMLTextAreaElement) => {
    if (preRef.current) {
      preRef.current.scrollTop = el.scrollTop;
      preRef.current.scrollLeft = el.scrollLeft;
    }
  }, []);

  // (sq-rcuvq) Merge editor hotkeys (Ctrl+/, Tab, Shift+Tab) with any parent-provided handler.
  // Turtle/SHACL/N3 comments also use '#'. The parent handler (e.g. Ctrl+Enter → validate) is
  // always called after so its own keys still fire.
  const mergedKeyDown = React.useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      handleEditorKeyDown(e, "#", onChange);
      onKeyDown?.(e);
    },
    [onChange, onKeyDown],
  );

  const tokens = React.useMemo(() => tokenizeTurtle(value), [value]);

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
            `spellCheck` off — Turtle is not prose. */}
        <textarea
          id={id}
          aria-label={ariaLabel}
          value={value}
          placeholder={placeholder}
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
    </div>
  );
}
