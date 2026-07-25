// (sq-rcuvq) [SONNET-4.6] — unit tests for editor-keys.ts pure transforms.
//
// Covers: applyCommentToggle, applyTabIndent, applyShiftTabDedent.
// handleEditorKeyDown is not unit-tested here (it wires into the DOM/React lifecycle);
// its behaviour is exercised by the Playwright e2e spec (editor-hotkeys.spec.ts).
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  applyCommentToggle,
  applyTabIndent,
  applyShiftTabDedent,
} from "./editor-keys.js";

// ---------------------------------------------------------------------------
// applyCommentToggle
// ---------------------------------------------------------------------------

test("applyCommentToggle – adds '# ' to a single uncommented line (caret anywhere on line)", () => {
  const text = "SELECT * WHERE { ?s ?p ?o }";
  const result = applyCommentToggle({ text, selStart: 7, selEnd: 7 }, "#");
  assert.equal(result.text, "# SELECT * WHERE { ?s ?p ?o }");
});

test("applyCommentToggle – removes '# ' from a single commented line", () => {
  const text = "# SELECT * WHERE { ?s ?p ?o }";
  const result = applyCommentToggle({ text, selStart: 9, selEnd: 9 }, "#");
  assert.equal(result.text, "SELECT * WHERE { ?s ?p ?o }");
});

test("applyCommentToggle – caret at column 0 of uncommented line", () => {
  const text = "hello";
  const result = applyCommentToggle({ text, selStart: 0, selEnd: 0 }, "#");
  assert.equal(result.text, "# hello");
  // Cursor should move right by 2 (past the added '# ')
  assert.equal(result.selStart, 2);
  assert.equal(result.selEnd, 2);
});

test("applyCommentToggle – caret at column 0 of commented line (remove)", () => {
  const text = "# hello";
  const result = applyCommentToggle({ text, selStart: 0, selEnd: 0 }, "#");
  assert.equal(result.text, "hello");
  // After removing '# ', caret clamps to line start (position 0)
  assert.equal(result.selStart, 0);
  assert.equal(result.selEnd, 0);
});

test("applyCommentToggle – multi-line: adds '# ' to all lines when none commented", () => {
  const text = "line1\nline2\nline3";
  // Select across all three lines
  const result = applyCommentToggle({ text, selStart: 0, selEnd: text.length }, "#");
  assert.equal(result.text, "# line1\n# line2\n# line3");
});

test("applyCommentToggle – multi-line: removes '# ' when ALL lines commented", () => {
  const text = "# line1\n# line2\n# line3";
  const result = applyCommentToggle({ text, selStart: 0, selEnd: text.length }, "#");
  assert.equal(result.text, "line1\nline2\nline3");
});

test("applyCommentToggle – mixed: adds '# ' to ALL lines when not all commented", () => {
  const text = "# line1\nline2\n# line3";
  const result = applyCommentToggle({ text, selStart: 0, selEnd: text.length }, "#");
  assert.equal(result.text, "# # line1\n# line2\n# # line3");
});

test("applyCommentToggle – partial selection only covers a subset of lines", () => {
  const text = "line1\nline2\nline3";
  // Selection only touches line2 (positions 6 to 10)
  const result = applyCommentToggle({ text, selStart: 6, selEnd: 10 }, "#");
  assert.equal(result.text, "line1\n# line2\nline3");
});

test("applyCommentToggle – selection ending at start of next line excludes that line", () => {
  const text = "line1\nline2\nline3";
  // selEnd = 6 means the selection ends at the start of line2 (after line1's \n)
  const result = applyCommentToggle({ text, selStart: 0, selEnd: 6 }, "#");
  // Only line1 should be commented (selEnd points to char after '\n', so line2 is excluded)
  assert.equal(result.text, "# line1\nline2\nline3");
});

test("applyCommentToggle – empty line in block does not prevent adding comments", () => {
  const text = "line1\n\nline3";
  const result = applyCommentToggle({ text, selStart: 0, selEnd: text.length }, "#");
  // Empty line gets '# ' too (it was not counted in nonEmpty check, so allCommented is false for
  // the non-empty lines when none start with '# ')
  assert.equal(result.text, "# line1\n# \n# line3");
});

test("applyCommentToggle – cursor position advances by 2 when adding on non-first line", () => {
  const text = "line1\nline2";
  // Caret at start of line2 (position 6)
  const result = applyCommentToggle({ text, selStart: 6, selEnd: 6 }, "#");
  assert.equal(result.text, "line1\n# line2");
  // line2 is the second line of the block (bStart = 6 = lineStart of line2)
  // linesBeforeCaret for selStart=6 within block starting at 6: slice is empty → 1 line
  // new caret = 6 + 2*1 = 8
  assert.equal(result.selStart, 8);
});

// ---------------------------------------------------------------------------
// applyTabIndent
// ---------------------------------------------------------------------------

test("applyTabIndent – inserts 2 spaces at cursor", () => {
  const text = "hello";
  const result = applyTabIndent({ text, selStart: 2, selEnd: 2 });
  assert.equal(result.text, "he  llo");
  assert.equal(result.selStart, 4);
  assert.equal(result.selEnd, 4);
});

test("applyTabIndent – inserts at position 0", () => {
  const text = "hello";
  const result = applyTabIndent({ text, selStart: 0, selEnd: 0 });
  assert.equal(result.text, "  hello");
  assert.equal(result.selStart, 2);
});

test("applyTabIndent – inserts at end of text", () => {
  const text = "hello";
  const result = applyTabIndent({ text, selStart: 5, selEnd: 5 });
  assert.equal(result.text, "hello  ");
  assert.equal(result.selStart, 7);
});

test("applyTabIndent – guard: returns unchanged state when selection is non-empty", () => {
  const text = "hello";
  const result = applyTabIndent({ text, selStart: 1, selEnd: 3 });
  assert.deepEqual(result, { text: "hello", selStart: 1, selEnd: 3 });
});

// ---------------------------------------------------------------------------
// applyShiftTabDedent
// ---------------------------------------------------------------------------

test("applyShiftTabDedent – removes 2 leading spaces from caret line (no selection)", () => {
  const text = "  hello";
  const result = applyShiftTabDedent({ text, selStart: 3, selEnd: 3 });
  assert.equal(result.text, "hello");
  assert.equal(result.selStart, 1);
  assert.equal(result.selEnd, 1);
});

test("applyShiftTabDedent – removes only 1 space when only 1 is present (no selection)", () => {
  const text = " hello";
  const result = applyShiftTabDedent({ text, selStart: 2, selEnd: 2 });
  assert.equal(result.text, "hello");
  assert.equal(result.selStart, 1);
});

test("applyShiftTabDedent – no-op when line has no leading spaces (no selection)", () => {
  const text = "hello";
  const result = applyShiftTabDedent({ text, selStart: 2, selEnd: 2 });
  assert.deepEqual(result, { text: "hello", selStart: 2, selEnd: 2 });
});

test("applyShiftTabDedent – caret at column 0 moves to line start after removal", () => {
  const text = "  hello";
  const result = applyShiftTabDedent({ text, selStart: 0, selEnd: 0 });
  assert.equal(result.text, "hello");
  // selStart - toRemove = 0 - 2 = -2 → clamp to 0
  assert.equal(result.selStart, 0);
});

test("applyShiftTabDedent – dedents every selected line (multi-line selection)", () => {
  const text = "  line1\n  line2\n  line3";
  const result = applyShiftTabDedent({ text, selStart: 0, selEnd: text.length });
  assert.equal(result.text, "line1\nline2\nline3");
});

test("applyShiftTabDedent – dedents partially-indented lines (only removes available spaces)", () => {
  const text = " line1\n  line2";
  const result = applyShiftTabDedent({ text, selStart: 0, selEnd: text.length });
  assert.equal(result.text, "line1\nline2");
});

test("applyShiftTabDedent – column-0 lines in multi-line selection are left unchanged", () => {
  const text = "  line1\nline2";
  const result = applyShiftTabDedent({ text, selStart: 0, selEnd: text.length });
  // line1 loses 2 spaces; line2 has none → no change
  assert.equal(result.text, "line1\nline2");
});

test("applyShiftTabDedent – multi-line selection: result selects the whole modified block", () => {
  const text = "  a\n  b";
  const result = applyShiftTabDedent({ text, selStart: 0, selEnd: text.length });
  assert.equal(result.text, "a\nb");
  assert.equal(result.selStart, 0);
  assert.equal(result.selEnd, result.text.length);
});
