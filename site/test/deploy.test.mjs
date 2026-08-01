// [GPT-5.6] sq-44ga1 — pin the provider/button and secure-default matrices rendered by /deploy.

import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  DEMO_ENVIRONMENT,
  DEPLOY_OPTIONS,
  ONE_CLICK_BUTTONS,
  OPEN_BY_DEFAULT_CAVEAT,
  SECURE_DEFAULTS,
} from "../src/app/deploy/deploy-options.ts";

test("deploy surface covers every provider family in architecture order", () => {
  assert.deepEqual(
    DEPLOY_OPTIONS.map((option) => option.id),
    ["aws", "azure", "gcp", "fly", "render", "railway", "terraform", "helm"],
  );
  assert.ok(DEPLOY_OPTIONS.every((option) => option.docsHref.startsWith("https://")));
  assert.ok(DEPLOY_OPTIONS.every((option) => option.security.length > 40));
});

test("every checked-in one-click button is wired for both server targets", () => {
  assert.deepEqual(
    ONE_CLICK_BUTTONS.map(({ provider, target }) => `${provider}:${target}`),
    [
      "azure:sparq-server",
      "azure:solid-lws",
      "fly:sparq-server",
      "fly:solid-lws",
      "render:sparq-server",
      "render:solid-lws",
      "railway:sparq-server",
      "railway:solid-lws",
    ],
  );
  assert.ok(ONE_CLICK_BUTTONS.every((button) => button.href.startsWith("https://")));
});

test("AWS and GCP do not invent unsupported public launch buttons", () => {
  for (const id of ["aws", "gcp"]) {
    const option = DEPLOY_OPTIONS.find((candidate) => candidate.id === id);
    assert.ok(option);
    assert.deepEqual(option.buttons, []);
    assert.match(option.caveat, /does not|cannot/);
    assert.ok(option.command);
  }
});

// [OPUS-5] sq-cepjb — the /deploy demo section: its two links must resolve to assets that
// exist on main, and its banner must keep stating the throwaway posture.
test("demo section links checked-in assets and advertises no hosted instance", () => {
  const REPO_BLOB = "https://github.com/sparq-org/sparq/blob/main/";
  for (const href of [
    DEMO_ENVIRONMENT.manifestsHref,
    DEMO_ENVIRONMENT.designHref,
  ]) {
    assert.ok(href.startsWith(REPO_BLOB), `not a repo blob link: ${href}`);
    // The demo has no sparq-hosted deployment; linking a *.run.app URL from this page
    // would claim one exists. Flip this deliberately if that ever changes.
    assert.ok(!href.includes("run.app"), `unexpected hosted URL: ${href}`);
    const repoPath = href.slice(REPO_BLOB.length);
    const onDisk = fileURLToPath(new URL(`../../${repoPath}`, import.meta.url));
    assert.ok(existsSync(onDisk), `demo link points at a missing file: ${repoPath}`);
  }
});

test("demo caveats state the throwaway, shared, wipe-on-idle posture", () => {
  assert.deepEqual(
    DEMO_ENVIRONMENT.caveats.map(({ rule }) => rule),
    ["Throwaway identities", "No isolation between visitors", "Wiped when idle"],
  );
  assert.ok(DEMO_ENVIRONMENT.caveats.every(({ detail }) => detail.length > 80));
  const [identities, isolation, wipe] = DEMO_ENVIRONMENT.caveats;
  assert.match(identities.detail, /unverified/);
  assert.match(isolation.detail, /anonymous writes are refused/);
  assert.match(wipe.detail, /no guaranteed wipe deadline/);
});

test("secure-default copy pins the image-layer caveat and operational controls", () => {
  assert.equal(
    OPEN_BY_DEFAULT_CAVEAT,
    "sparq-server is open-by-default at the image layer; these templates gate it with a token — do not remove the token wiring.",
  );
  assert.deepEqual(
    SECURE_DEFAULTS.map(({ rule }) => rule),
    ["Auth on", "HTTPS only", "Secrets stay secret", "Server-specific health"],
  );
});
