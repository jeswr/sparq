"use client";

// [FABLE-5] sq-ymr2e.9 — shared WAI-ARIA tabs keyboard contract for the site's
// `role="tablist"` selector family (home hero Query/Data selectors, and other tab widgets).
//
// The site's tab widgets already carried the STATIC half of the ARIA Authoring
// Practices tabs pattern (`tablist`/`tab` roles + `aria-selected`) but not the
// BEHAVIORAL half: a roving tabindex (exactly one tab in the page Tab-order) and
// Left/Right-arrow + Home/End focus movement. That gap is exactly the "checks axe
// cannot see" class research/web-gui-test-program.md §3.2 gates on, and it is
// asserted end-to-end by e2e/a11y-keyboard.spec.ts — strip this behaviour and that
// spec goes red (its non-vacuity guard proves the assertions are live).
//
// Contract implemented (APG "Tabs Pattern", automatic activation — every panel
// switch here is cheap local state, so selection follows focus):
//   * ArrowRight / ArrowLeft move focus to the next / previous ENABLED tab,
//     wrapping at the ends; Home / End jump to the first / last enabled tab.
//   * The newly focused tab is ACTIVATED (a programmatic click on the tab button,
//     the same code path as a keyboard Enter/Space on it).
//   * `rovingTabIndex(selected)` puts only the selected tab in the Tab-order
//     (tabIndex 0, all siblings -1), so Tab from the tablist moves INTO the panel
//     content, never through every tab.
//
// Dependency-free (a keydown handler + a tabIndex helper); works on the plain
// <button role="tab"> markup every widget in the family already renders.
import * as React from "react";

/** Keys the tablist handles; everything else falls through untouched. */
const TABLIST_KEYS = ["ArrowRight", "ArrowLeft", "Home", "End"] as const;
type TablistKey = (typeof TABLIST_KEYS)[number];

function isTablistKey(key: string): key is TablistKey {
  return (TABLIST_KEYS as readonly string[]).includes(key);
}

/** The enabled `role="tab"` buttons of the tablist, in DOM order. */
function enabledTabs(tablist: HTMLElement): HTMLElement[] {
  return Array.from(
    tablist.querySelectorAll<HTMLElement>('[role="tab"]'),
  ).filter(
    (tab) =>
      !(tab as HTMLButtonElement).disabled &&
      tab.getAttribute("aria-disabled") !== "true",
  );
}

/**
 * The roving-tabindex value for one tab: only the SELECTED tab participates in the
 * page Tab-order (APG tabs pattern), so keyboard users Tab past the widget in one
 * stop and use arrow keys within it.
 */
export function rovingTabIndex(selected: boolean): 0 | -1 {
  return selected ? 0 : -1;
}

/**
 * Arrow-key roving-focus handler for a `role="tablist"` container whose tabs are
 * `<button role="tab">` children. Attach as the tablist's `onKeyDown`. Focus moves
 * to the target tab and activates it (automatic activation).
 */
export function useRovingTablist(): {
  onKeyDown: (event: React.KeyboardEvent<HTMLElement>) => void;
} {
  const onKeyDown = React.useCallback(
    (event: React.KeyboardEvent<HTMLElement>) => {
      if (!isTablistKey(event.key)) return;
      const tabs = enabledTabs(event.currentTarget);
      const current = tabs.indexOf(event.target as HTMLElement);
      if (current === -1 || tabs.length < 2) return;
      event.preventDefault();

      let next: number;
      switch (event.key) {
        case "ArrowRight":
          next = (current + 1) % tabs.length;
          break;
        case "ArrowLeft":
          next = (current - 1 + tabs.length) % tabs.length;
          break;
        case "Home":
          next = 0;
          break;
        default: // "End"
          next = tabs.length - 1;
      }

      const target = tabs[next];
      target.focus();
      // Automatic activation: selection follows focus (cheap local-state panels).
      target.click();
    },
    [],
  );

  return { onKeyDown };
}
