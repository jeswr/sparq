// [OPUS-4.8] sq-atb0 — unit tests for the persistent cross-session WORKSPACE model + the
// persistence abstraction (`@sparq/client` `workspace.ts`): the data model + (de)serialisation
// guards, the web (localStorage) backend over an in-memory polyfill, the in-memory backend, the
// Tauri backend over an in-memory fake filesystem, and the runtime backend-selection factory.
// These cover the pure, framework-free logic the REPL workspace panel relies on. Run via
// `npm run test:unit`.
//
// Imported by RELATIVE path (not the `@sparq/client` alias) so the build-free ts-loader resolves
// it. In Node there is no `window`/`localStorage`, so the tests inject the storage surfaces
// explicitly — exactly the dependency-injection seams the abstraction is designed for.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  WORKSPACE_SCHEMA,
  WORKSPACE_INFERENCE_MODES,
  WebWorkspaceStore,
  MemoryWorkspaceStore,
  TauriWorkspaceStore,
  createWorkspaceStore,
  detectWebStorage,
  isTauriRuntime,
  newWorkspace,
  parseInferenceMode,
  parseWorkspace,
  workspaceId,
  workspaceSummary,
} from "../../packages/sparq-client/src/workspace.ts";

// A minimal in-memory localStorage polyfill (the `KeyValueStorage` shape).
function fakeKv() {
  const map = new Map();
  return {
    getItem: (k) => (map.has(k) ? map.get(k) : null),
    setItem: (k, v) => map.set(k, String(v)),
    removeItem: (k) => map.delete(k),
    _map: map,
  };
}

// A minimal in-memory TauriFsApi over a path→contents map.
function fakeFs() {
  const files = new Map();
  return {
    baseDir: 11, // AppLocalData placeholder
    _files: files,
    async readTextFile(path) {
      if (!files.has(path)) throw new Error("ENOENT");
      return files.get(path);
    },
    async writeTextFile(path, contents) {
      files.set(path, contents);
    },
    async remove(path) {
      files.delete(path);
    },
    async exists(path) {
      // Treat the dir marker generously: any path that is a known dir prefix "exists".
      if (files.has(path)) return true;
      return [...files.keys()].some((k) => k.startsWith(path + "/"));
    },
    async mkdir() {},
    async readDir() {
      return [];
    },
  };
}

// --- the data model ---------------------------------------------------------

test("newWorkspace seeds a current-schema record with a fresh id", () => {
  const ws = newWorkspace("Demo", "SELECT * WHERE { ?s ?p ?o }");
  assert.equal(ws.schema, WORKSPACE_SCHEMA);
  assert.equal(ws.name, "Demo");
  assert.equal(ws.editor.query, "SELECT * WHERE { ?s ?p ?o }");
  assert.equal(ws.editor.mode, "run");
  assert.equal(ws.editor.endpointActive, false);
  assert.deepEqual(ws.sources, []);
  assert.match(ws.id, /^ws_/);
});

test("workspaceId is unique across calls", () => {
  const a = workspaceId();
  const b = workspaceId();
  assert.notEqual(a, b);
});

test("workspaceSummary derives an approx triple count from the snapshot", () => {
  const ws = newWorkspace("S", "SELECT 1");
  ws.dataSnapshot = "<a> <b> <c> .\n<d> <e> <f> .";
  ws.sources = [{ kind: "url", label: "x", format: "turtle", importedAt: 1 }];
  const s = workspaceSummary(ws);
  assert.equal(s.approxTriples, 2);
  assert.equal(s.sourceCount, 1);
  assert.equal(workspaceSummary(newWorkspace("E", "")).approxTriples, 0);
});

test("parseWorkspace round-trips a valid record and rejects junk", () => {
  const ws = newWorkspace("RT", "ASK {}");
  ws.dataSnapshot = "<a> <b> <c> .";
  ws.sources = [
    { kind: "local", label: "f.ttl", format: "turtle", bytes: 12, importedAt: 5 },
    { kind: "url", label: "remote", url: "https://x/y.ttl", format: "turtle", importedAt: 6 },
  ];
  const parsed = parseWorkspace(JSON.parse(JSON.stringify(ws)));
  assert.ok(parsed);
  assert.equal(parsed.name, "RT");
  assert.equal(parsed.sources.length, 2);
  assert.equal(parsed.sources[1].url, "https://x/y.ttl");
  // junk / stale-schema / missing fields are rejected
  assert.equal(parseWorkspace(null), null);
  assert.equal(parseWorkspace({}), null);
  assert.equal(parseWorkspace({ ...ws, schema: 999 }), null);
  assert.equal(parseWorkspace({ ...ws, editor: undefined }), null);
});

test("parseWorkspace drops malformed sources but keeps valid ones", () => {
  const ws = newWorkspace("M", "SELECT 1");
  const raw = JSON.parse(JSON.stringify(ws));
  raw.sources = [
    { kind: "bogus" },
    { kind: "url", label: "ok", format: "turtle", importedAt: 1 },
    null,
  ];
  const parsed = parseWorkspace(raw);
  assert.equal(parsed.sources.length, 1);
  assert.equal(parsed.sources[0].label, "ok");
});

test("parseWorkspace never persists a token field (editor carries no secret)", () => {
  const ws = newWorkspace("T", "SELECT 1");
  ws.editor.endpointActive = true;
  ws.editor.endpointUrl = "https://srv/sparql";
  const parsed = parseWorkspace(JSON.parse(JSON.stringify(ws)));
  assert.equal(parsed.editor.endpointUrl, "https://srv/sparql");
  assert.ok(!("token" in parsed.editor));
});

// --- the web (localStorage) backend ----------------------------------------

test("WebWorkspaceStore: save → list → load → setLast → remove", async () => {
  const store = new WebWorkspaceStore(fakeKv());
  assert.equal(store.backend, "web");
  assert.deepEqual(await store.list(), []);

  const a = newWorkspace("A", "SELECT 1");
  const b = newWorkspace("B", "SELECT 2");
  await store.save(a);
  // A 2 ms gap so `b.updatedAt` is strictly later than `a.updatedAt` even on a coarse clock,
  // making the most-recently-updated-first ordering deterministic for the assertion below.
  await new Promise((r) => setTimeout(r, 2));
  await store.save(b);

  const list = await store.list();
  assert.equal(list.length, 2);
  // most-recently-updated first (b saved after a)
  assert.equal(list[0].id, b.id);

  const loaded = await store.load(a.id);
  assert.equal(loaded.name, "A");

  await store.setLastOpenedId(a.id);
  assert.equal(await store.lastOpenedId(), a.id);

  await store.remove(a.id);
  assert.equal(await store.load(a.id), null);
  assert.equal((await store.list()).length, 1);
  // last-opened pointer is cleared when the workspace it pointed at is deleted
  assert.equal(await store.lastOpenedId(), null);
});

test("WebWorkspaceStore: lastOpenedId is null when it points at a missing id", async () => {
  const kv = fakeKv();
  const store = new WebWorkspaceStore(kv);
  await store.setLastOpenedId("ws_ghost");
  assert.equal(await store.lastOpenedId(), null);
});

test("WebWorkspaceStore: a corrupt record is skipped, not thrown", async () => {
  const kv = fakeKv();
  const store = new WebWorkspaceStore(kv);
  const a = newWorkspace("Good", "SELECT 1");
  await store.save(a);
  // Hand-corrupt a second indexed entry.
  kv.setItem("sparq.workspace.v1.__index__", JSON.stringify([a.id, "ws_corrupt"]));
  kv.setItem("sparq.workspace.v1.ws_corrupt", "{not json");
  const list = await store.list();
  assert.equal(list.length, 1);
  assert.equal(list[0].name, "Good");
  assert.equal(await store.load("ws_corrupt"), null);
});

// --- the in-memory backend --------------------------------------------------

test("MemoryWorkspaceStore behaves like the web backend (session-only)", async () => {
  const store = new MemoryWorkspaceStore();
  assert.equal(store.backend, "memory");
  const a = newWorkspace("A", "SELECT 1");
  await store.save(a);
  await store.setLastOpenedId(a.id);
  assert.equal((await store.list()).length, 1);
  assert.equal(await store.lastOpenedId(), a.id);
  await store.remove(a.id);
  assert.equal(await store.lastOpenedId(), null);
});

// --- the Tauri backend (over a fake fs) ------------------------------------

test("TauriWorkspaceStore: save → list → load → remove over a fake fs", async () => {
  const fs = fakeFs();
  const store = new TauriWorkspaceStore(fs);
  assert.equal(store.backend, "tauri");

  const a = newWorkspace("Disk A", "SELECT 1");
  a.dataSnapshot = "<a> <b> <c> .";
  await store.save(a);
  await store.setLastOpenedId(a.id);

  // The record + index + last pointer are real files under workspaces/.
  assert.ok(fs._files.has(`workspaces/${a.id}.json`));
  assert.ok(fs._files.has("workspaces/__index__.json"));

  const list = await store.list();
  assert.equal(list.length, 1);
  assert.equal(list[0].approxTriples, 1);

  const loaded = await store.load(a.id);
  assert.equal(loaded.name, "Disk A");
  assert.equal(await store.lastOpenedId(), a.id);

  await store.remove(a.id);
  assert.equal(await store.load(a.id), null);
  assert.equal(await store.lastOpenedId(), null);
});

// --- runtime backend selection ---------------------------------------------

test("isTauriRuntime is false in a headless Node (no window)", () => {
  assert.equal(isTauriRuntime(), false);
});

test("detectWebStorage returns null when localStorage is absent", () => {
  // Node has no localStorage global → null (the memory fallback path).
  assert.equal(detectWebStorage(), null);
});

test("createWorkspaceStore falls back to memory with no storage + no Tauri", async () => {
  const store = await createWorkspaceStore();
  assert.equal(store.backend, "memory");
});

test("createWorkspaceStore ignores a Tauri fs loader outside a Tauri runtime", async () => {
  let called = false;
  const store = await createWorkspaceStore({
    loadTauriFs: async () => {
      called = true;
      return fakeFs();
    },
  });
  // Not in a Tauri webview → the loader is never invoked, and it degrades to memory.
  assert.equal(called, false);
  assert.equal(store.backend, "memory");
});

// --- sq-glo5r: N3 inference mode + rulesDocs --------------------------------

test("parseInferenceMode recognises 'n3'", () => {
  assert.equal(parseInferenceMode("n3"), "n3");
});

test("WORKSPACE_INFERENCE_MODES includes 'n3'", () => {
  assert.ok(WORKSPACE_INFERENCE_MODES.includes("n3"));
});

test("parseWorkspace round-trips a workspace with rulesDocs", () => {
  const ws = newWorkspace("N3-test", "SELECT 1");
  ws.inference = "n3";
  ws.rulesDocs = [
    { name: "my-rule", text: "{ ?s a ex:A } => { ?s a ex:B . }", addedAt: 123456 },
    { name: "disabled-rule", text: "{ ?x a ex:C } => { ?x a ex:D . }", addedAt: 123457, enabled: false },
  ];
  const parsed = parseWorkspace(JSON.parse(JSON.stringify(ws)));
  assert.ok(parsed);
  assert.equal(parsed.inference, "n3");
  assert.equal(parsed.rulesDocs.length, 2);
  assert.equal(parsed.rulesDocs[0].name, "my-rule");
  assert.equal(parsed.rulesDocs[0].addedAt, 123456);
  assert.equal(parsed.rulesDocs[0].enabled, undefined); // enabled=true is stored as absent
  assert.equal(parsed.rulesDocs[1].enabled, false);
});

test("parseWorkspace defaults to empty rulesDocs when the field is absent (old record)", () => {
  const ws = newWorkspace("old-record", "SELECT 1");
  const raw = JSON.parse(JSON.stringify(ws));
  delete raw.rulesDocs; // simulate a record from before sq-glo5r
  const parsed = parseWorkspace(raw);
  assert.ok(parsed);
  assert.deepEqual(parsed.rulesDocs, []);
});

test("parseWorkspace skips malformed rulesDocs entries but keeps valid ones", () => {
  const ws = newWorkspace("M", "SELECT 1");
  const raw = JSON.parse(JSON.stringify(ws));
  raw.rulesDocs = [
    { name: "good", text: "...", addedAt: 1 },
    { name: "no-text", addedAt: 2 }, // missing text → skipped
    null, // null → skipped
    { name: "no-addedAt", text: "..." }, // missing addedAt → skipped
  ];
  const parsed = parseWorkspace(raw);
  assert.ok(parsed);
  assert.equal(parsed.rulesDocs.length, 1);
  assert.equal(parsed.rulesDocs[0].name, "good");
});
