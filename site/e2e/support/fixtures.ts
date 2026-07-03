// [OPUS-4.8] sq-ymr2e.1 — the EXTENDED Playwright `test` every site spec should import.
//
// Design of record: research/web-gui-test-program.md §1. It composes two AUTO fixtures so a spec
// is hermetic + deterministic just by importing this `test` (no per-spec boilerplate):
//   * hermetic  — installs the hermetic-network route on the context (block external, fixture-route
//                 the github releases call). Auto, but also nameable so a spec can assert on the
//                 recorded `externalRequests` (the zero-network invariant, §2.1).
//   * determinism — frozen clock + seeded random + animations-off on the page (see determinism.ts).
//
// It also declares two OPTIONS a spec tunes via `test.use({ ... })`:
//   * github     — which releases/latest fixture to serve ("release" | "notFound" | "error" | "off").
//   * allowHosts — extra external hosts to allow (keep small; every entry is a hermeticity hole).
//
// Config-level determinism (viewport / dark colour-scheme / reduced-motion / UTC / en-US) lives in
// playwright.config.ts because those are context options that apply to EVERY spec, migrated or not.
import { test as base, expect } from "@playwright/test";
import { applyDeterminism } from "./determinism";
import {
  DEFAULT_HERMETIC_POLICY,
  installHermeticNetwork,
  type HermeticController,
} from "./hermetic-net";
import type { GithubFixtureName } from "./test-data/github";

type HermeticOptions = {
  /** Install the hermetic-network route at all. Default: true. Set false for specs whose runtime
   *  is incompatible with blanket `context.route` interception — notably the ZK prover, whose
   *  bb.js SharedArrayBuffer multithreading needs the coi-serviceworker's cross-origin isolation
   *  (COOP/COEP header injection), which interception disrupts. Such specs keep the DETERMINISM
   *  harness (frozen clock, seeded random, animations-off) but skip the network block; they are
   *  same-origin, in-browser, zero-network by construction, so the block would only be a no-op. */
  hermeticNetwork: boolean;
  /** Which releases/latest fixture the hermetic layer serves. Default: "release". */
  github: GithubFixtureName;
  /** Extra external hosts to allow through the hermetic block. Default: none. */
  allowHosts: (string | RegExp)[];
};

type HermeticFixtures = {
  hermetic: HermeticController;
  determinism: void;
};

const INERT_CONTROLLER: HermeticController = {
  externalRequests: [],
  routedRequests: [],
  reset() {},
};

export const test = base.extend<HermeticOptions & HermeticFixtures>({
  // ── Options (override with test.use({ hermeticNetwork, github, allowHosts })). ──────────────
  hermeticNetwork: [true, { option: true }],
  github: [DEFAULT_HERMETIC_POLICY.github, { option: true }],
  allowHosts: [DEFAULT_HERMETIC_POLICY.allowHosts, { option: true }],

  // ── Auto fixtures. ─────────────────────────────────────────────────────────────────────────
  hermetic: [
    async ({ context, hermeticNetwork, github, allowHosts }, use) => {
      const controller = hermeticNetwork
        ? await installHermeticNetwork(context, { github, allowHosts })
        : INERT_CONTROLLER;
      await use(controller);
    },
    { auto: true },
  ],
  determinism: [
    async ({ page }, use) => {
      await applyDeterminism(page);
      await use();
    },
    { auto: true },
  ],
});

export { expect };
export type { HermeticController };
