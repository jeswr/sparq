// (sq-ds9pg) [FABLE-5] — regression test for the workspace rename/delete action-group
// focus-visibility contract (workspace-actions-visibility.ts).
//
// THE REGRESSION (GPT-5.6 GUI review, adversarially verified): the action group was hidden with
// `opacity-0 … group-hover:opacity-100` ONLY, with no reveal on keyboard focus. `opacity-0` keeps
// the buttons in the tab order, so a keyboard-only user tabbed onto fully-transparent rename/delete
// buttons with no visible focus indicator — a WCAG 2.1 AA 2.4.7 (Focus Visible) failure.
//
// No jsdom is available in this build-free node:test harness, so we assert the VISIBILITY CONTRACT
// directly on the classnames the component actually renders (they are imported from the same
// module the component imports — see workspace-switcher.tsx — so this cannot drift from the markup).
// The load-bearing assertion: the container reveals on keyboard focus (`group-focus-within`) as
// well as pointer hover, and each button carries a visible focus ring. Reverting the fix (dropping
// `group-focus-within:opacity-100`) makes this go red.
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  WORKSPACE_ACTIONS_CONTAINER_CLASS,
  WORKSPACE_RENAME_BUTTON_CLASS,
  WORKSPACE_DELETE_BUTTON_CLASS,
} from "./workspace-actions-visibility.js";

test("action group is revealed on KEYBOARD FOCUS, not just pointer hover (WCAG 2.4.7)", () => {
  const classes = WORKSPACE_ACTIONS_CONTAINER_CLASS.split(/\s+/);

  // It starts hidden…
  assert.ok(classes.includes("opacity-0"), "the group is opacity-0 by default");
  // …revealed on pointer hover (unchanged)…
  assert.ok(
    classes.includes("group-hover:opacity-100"),
    "the group must reveal on pointer hover",
  );
  // …AND — the fix — revealed when either inner button receives keyboard focus.
  assert.ok(
    classes.includes("group-focus-within:opacity-100"),
    "REGRESSION sq-ds9pg: the group MUST reveal on keyboard focus (group-focus-within) — " +
      "opacity-0 alone leaves the buttons tabbable but invisible (WCAG 2.4.7 failure)",
  );
});

test("each action button self-reveals on focus and shows a visible focus ring", () => {
  for (const [name, cls] of [
    ["rename", WORKSPACE_RENAME_BUTTON_CLASS],
    ["delete", WORKSPACE_DELETE_BUTTON_CLASS],
  ] as const) {
    const classes = cls.split(/\s+/);
    assert.ok(
      classes.includes("focus-visible:opacity-100"),
      `${name} button must become visible on focus even outside the group reveal`,
    );
    assert.ok(
      classes.includes("focus-visible:ring-2"),
      `${name} button must render a visible focus ring (2.4.7 focus indicator)`,
    );
    assert.ok(
      classes.includes("focus-visible:outline-none"),
      `${name} button replaces the default outline with the ring cleanly`,
    );
  }
});
