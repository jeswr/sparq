// [GPT-5.6] sq-44ga1 — pin the provider/button and secure-default matrices rendered by /deploy.

import assert from "node:assert/strict";
import { test } from "node:test";

import {
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
