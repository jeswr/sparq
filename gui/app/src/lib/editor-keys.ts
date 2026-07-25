// (sq-rcuvq) [SONNET-4.6] — Editor hotkeys: Ctrl+/ comment toggle, Tab indent, Shift+Tab dedent.
//
// Pure string-transform helpers so tests need no DOM / React setup, plus a thin handler that
// applies them to a React-controlled <textarea> via requestAnimationFrame.
//
// Exports
//   EditorState               — {text, selStart, selEnd} shape used by all transforms
//   applyCommentToggle        — toggle `commentChar + ' '` prefix on affected lines
//   applyTabIndent            — insert 2 spaces at caret (no selection)
//   applyShiftTabDedent       — remove up to 2 leading spaces per affected line
//   EditorKeyEvent            — minimal key-event interface (compatible with React.KeyboardEvent)
//   handleEditorKeyDown       — wires the three hotkeys into a textarea onKeyDown

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface EditorState {
  text: string;
  /** Inclusive start of selection, or caret position when equal to selEnd. */
  selStart: number;
  /** Exclusive end of selection, or caret position when equal to selStart. */
  selEnd: number;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/** Absolute position of the start of the line that contains `pos`. */
function lineStartOf(text: string, pos: number): number {
  const i = text.lastIndexOf("\n", pos - 1);
  return i === -1 ? 0 : i + 1;
}

/** Absolute position of the end of the line that contains `pos` (before the trailing `\n`). */
function lineEndOf(text: string, pos: number): number {
  const i = text.indexOf("\n", pos);
  return i === -1 ? text.length : i;
}

// ---------------------------------------------------------------------------
// applyCommentToggle
// ---------------------------------------------------------------------------

/**
 * Toggle `commentChar + ' '` line-comment prefix on all lines that overlap the selection
 * (or the caret line when there is no selection).
 *
 * - If ALL non-empty affected lines already carry the prefix → remove it from every line.
 * - Otherwise → add the prefix to every line.
 *
 * Cursor/selection is adjusted so it tracks the same text content after the transform.
 */
export function applyCommentToggle(
  state: EditorState,
  commentChar: string,
): EditorState {
  const { text, selStart, selEnd } = state;
  const prefix = commentChar + " ";
  const P = prefix.length;

  // Block boundaries: full lines spanning the selection (or just the caret line).
  const bStart = lineStartOf(text, selStart);
  // If the selection ends exactly at a line-start (the \n was selected), exclude that last line.
  const effEnd =
    selEnd > selStart && text[selEnd - 1] === "\n" ? selEnd - 1 : selEnd;
  const bEnd = lineEndOf(text, effEnd);

  const block = text.slice(bStart, bEnd);
  const lines = block.split("\n");

  const nonEmpty = lines.filter((l) => l.trim() !== "");
  const allCommented =
    nonEmpty.length > 0 && nonEmpty.every((l) => l.startsWith(prefix));

  const newLines = allCommented
    ? lines.map((l) => (l.startsWith(prefix) ? l.slice(P) : l))
    : lines.map((l) => prefix + l);

  const newBlock = newLines.join("\n");
  const newText = text.slice(0, bStart) + newBlock + text.slice(bEnd);

  // Adjust an absolute position `pos` in [bStart, bEnd] after the transform.
  //
  // For a position on line index `li` (0-indexed from bStart), all of lines 0..li have
  // had ±P characters prepended, so the total shift is ±P * (li + 1).  When removing and
  // the shift would go past bStart we clamp to bStart.
  function adjustPos(pos: number): number {
    if (pos < bStart) return pos;
    if (pos >= bEnd) return pos + (newBlock.length - block.length);
    const countLines = text.slice(bStart, pos).split("\n").length; // li + 1
    const delta = allCommented ? -P : P;
    return Math.max(bStart, pos + delta * countLines);
  }

  if (selStart === selEnd) {
    const newCaret = adjustPos(selStart);
    return { text: newText, selStart: newCaret, selEnd: newCaret };
  }
  return {
    text: newText,
    selStart: adjustPos(selStart),
    selEnd: adjustPos(selEnd),
  };
}

// ---------------------------------------------------------------------------
// applyTabIndent
// ---------------------------------------------------------------------------

/**
 * Insert two spaces at the caret.
 * Only called when there is no selection (`selStart === selEnd`); returns state unchanged if
 * called with a selection (defensive guard).
 */
export function applyTabIndent(state: EditorState): EditorState {
  const { text, selStart, selEnd } = state;
  if (selStart !== selEnd) return state;
  const newText = text.slice(0, selStart) + "  " + text.slice(selStart);
  return { text: newText, selStart: selStart + 2, selEnd: selStart + 2 };
}

// ---------------------------------------------------------------------------
// applyShiftTabDedent
// ---------------------------------------------------------------------------

/**
 * Remove up to 2 leading spaces from:
 * - the caret line only (when `selStart === selEnd`), or
 * - every line that overlaps the selection.
 */
export function applyShiftTabDedent(state: EditorState): EditorState {
  const { text, selStart, selEnd } = state;

  if (selStart === selEnd) {
    // No selection: operate on caret line only.
    const ls = lineStartOf(text, selStart);
    let toRemove = 0;
    while (toRemove < 2 && text[ls + toRemove] === " ") toRemove++;
    if (toRemove === 0) return state;
    const newText = text.slice(0, ls) + text.slice(ls + toRemove);
    const newCaret = Math.max(ls, selStart - toRemove);
    return { text: newText, selStart: newCaret, selEnd: newCaret };
  }

  // Selection: operate on all overlapping lines.
  const bStart = lineStartOf(text, selStart);
  const effEnd = text[selEnd - 1] === "\n" ? selEnd - 1 : selEnd;
  const bEnd = lineEndOf(text, effEnd);

  const block = text.slice(bStart, bEnd);
  const lines = block.split("\n");
  const newLines = lines.map((l) => {
    let r = 0;
    while (r < 2 && l[r] === " ") r++;
    return l.slice(r);
  });
  const newBlock = newLines.join("\n");
  const newText = text.slice(0, bStart) + newBlock + text.slice(bEnd);

  // After a multi-line dedent, select the entire modified block.
  return {
    text: newText,
    selStart: bStart,
    selEnd: bStart + newBlock.length,
  };
}

// ---------------------------------------------------------------------------
// handleEditorKeyDown
// ---------------------------------------------------------------------------

/**
 * Minimal key-event interface compatible with `React.KeyboardEvent<HTMLTextAreaElement>`
 * and the native `KeyboardEvent`.  Written as a structural interface so unit tests can
 * construct it cheaply without a DOM.
 */
export interface EditorKeyEvent {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  preventDefault(): void;
  currentTarget: HTMLTextAreaElement;
}

/**
 * Wire into a `<textarea onKeyDown>`.  Handles:
 *
 * - Ctrl+/ (Cmd+/ on Mac): toggle line-comment using `commentChar + ' '`
 * - Tab (no selection): insert 2 spaces at cursor
 * - Shift+Tab: dedent (remove up to 2 leading spaces per affected line)
 *
 * Calls `onChange` to update React state, then uses `requestAnimationFrame` to restore
 * the computed selection AFTER React's controlled-input re-render.
 */
export function handleEditorKeyDown(
  e: EditorKeyEvent,
  commentChar: string,
  onChange: (value: string) => void,
): void {
  const ta = e.currentTarget;
  const state: EditorState = {
    text: ta.value,
    selStart: ta.selectionStart ?? 0,
    selEnd: ta.selectionEnd ?? 0,
  };

  let next: EditorState | null = null;

  if ((e.ctrlKey || e.metaKey) && e.key === "/") {
    next = applyCommentToggle(state, commentChar);
  } else if (e.key === "Tab" && !e.shiftKey && state.selStart === state.selEnd) {
    next = applyTabIndent(state);
  } else if (e.key === "Tab" && e.shiftKey) {
    next = applyShiftTabDedent(state);
  }

  if (next !== null) {
    e.preventDefault();
    onChange(next.text);
    const { selStart, selEnd } = next;
    requestAnimationFrame(() => ta.setSelectionRange(selStart, selEnd));
  }
}
