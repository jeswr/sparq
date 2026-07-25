// [OPUS-4.8] sq-ymr2e.1 — the shared BASE PAGE-OBJECT the sibling journey suites extend.
//
// Design of record: research/web-gui-test-program.md §1.3 — STABLE selectors only (getByRole /
// data-testid, never Tailwind classes). This centralises the app-shell affordances every surface
// shares (the slim top-bar nav, the ⌘K command palette) so a restyle updates ONE place, and the
// nav/palette locators stay role-based and churn-proof. Surface-specific page objects
// (e.g. HomeRunnerPage for sq-ymr2e.3) subclass this.
import { expect, type Locator, type Page } from "@playwright/test";
import { waitForAppReady } from "../app-ready";
import {
  expectRunnerState,
  type RunnerState,
  type RunnerStateOptions,
  type RunnerSurface,
} from "../runner-state";

export class BasePage {
  /** The palette shortcut modifier for the HOST OS the browser runs on (⌘ on macOS, Ctrl else). */
  static readonly paletteModifier = process.platform === "darwin" ? "Meta" : "Control";

  constructor(protected readonly page: Page) {}

  /** Navigate (route is relative to the /sparq/ baseURL) and wait for the app shell to be ready. */
  async goto(route: string): Promise<void> {
    await this.page.goto(route, { waitUntil: "domcontentloaded" });
    await waitForAppReady(this.page);
  }

  /** The slim top-bar primary navigation landmark. */
  primaryNav(): Locator {
    return this.page.getByRole("navigation", { name: "Primary" }).first();
  }

  /** A named destination link in the primary nav. */
  navLink(name: string): Locator {
    return this.primaryNav().getByRole("link", { name, exact: true });
  }

  /** The command-palette dialog (visible only when open). */
  commandPalette(): Locator {
    return this.page.getByRole("dialog", { name: "Command palette" });
  }

  /** Open the ⌘K/Ctrl-K command palette and assert it is visible. */
  async openCommandPalette(): Promise<Locator> {
    await this.page.keyboard.press(`${BasePage.paletteModifier}+KeyK`);
    const dialog = this.commandPalette();
    await expect(dialog).toBeVisible();
    return dialog;
  }

  /** Close the command palette with Escape and assert it is gone. */
  async closeCommandPalette(): Promise<void> {
    await this.page.keyboard.press("Escape");
    await expect(this.commandPalette()).toHaveCount(0);
  }

  /** Web-first wait for a runner surface to reach a UI state (never a timed sleep). */
  async expectRunnerState(
    surface: RunnerSurface,
    state: RunnerState,
    opts?: RunnerStateOptions,
  ): Promise<Locator> {
    return expectRunnerState(this.page, surface, state, opts);
  }
}
