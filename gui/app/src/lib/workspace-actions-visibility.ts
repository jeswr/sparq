// (sq-ds9pg) [FABLE-5] — the visibility contract for the workspace row's rename/delete action
// group (workspace-switcher.tsx).
//
// THE BUG THIS ENCODES A REGRESSION GUARD FOR (GPT-5.6 GUI review, adversarially verified): the
// action group was hidden with `opacity-0 … group-hover:opacity-100` ONLY. `opacity-0` does NOT
// remove an element from the tab order (unlike display:none / visibility:hidden / [hidden]), so a
// keyboard-only user tabbing through a workspace row landed focus on the (fully transparent)
// rename then delete buttons with no visible focus indicator — a WCAG 2.1 AA 2.4.7 (Focus
// Visible) failure and a blind-Enter delete-confirm mis-activation hazard.
//
// THE CONTRACT: the group MUST reveal on KEYBOARD FOCUS as well as pointer hover. Centralising the
// classnames here (a) keeps the component and its unit test reading the SAME strings, so the test
// cannot silently drift from the rendered markup, and (b) makes the a11y requirement explicit and
// regression-proof: dropping `group-focus-within:opacity-100` makes the group-visibility test go
// red (mutation-verified in workspace-actions-visibility.test.ts).

/**
 * Classes for the container that wraps the per-row rename + delete buttons.
 *
 * `group-hover:opacity-100`        — reveal on pointer hover (unchanged behaviour).
 * `group-focus-within:opacity-100` — reveal when EITHER button inside receives keyboard focus,
 *                                    fixing the 2.4.7 focus-visibility failure.
 */
export const WORKSPACE_ACTIONS_CONTAINER_CLASS =
  "flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity " +
  "group-hover:opacity-100 group-focus-within:opacity-100";

/** Shared focus-visible ring + self-reveal applied to each action button. */
const FOCUS_VISIBLE_BASE = "focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-2";

/** Classes for the rename button (neutral focus ring). */
export const WORKSPACE_RENAME_BUTTON_CLASS =
  "rounded p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground " +
  `${FOCUS_VISIBLE_BASE} focus-visible:ring-ring/50`;

/** Classes for the delete button (destructive focus ring). */
export const WORKSPACE_DELETE_BUTTON_CLASS =
  "rounded p-0.5 text-muted-foreground hover:bg-destructive/20 hover:text-destructive " +
  `${FOCUS_VISIBLE_BASE} focus-visible:ring-destructive/50`;
