// [OPUS-5] sq-3uzlf — unit tests for `pickFilesViaFileSystemAccess`, the File-returning half of
// the browser picker.
//
// Load-bearing property under test: the File System Access path hands back the RAW `File`
// objects, so a binary-aware caller (the RDF import pane's `readFilesWithDecompress`) can
// decompress an archive the same way it does for drag & drop. `pickTextFiles` reads text and
// therefore CANNOT serve that caller — hence the split.
//
// The `null` return is the other load-bearing contract: it means "File System Access is not
// usable here, use your own <input type=file>", and it must be distinguishable from a user
// cancel (which yields an EMPTY result, so the caller does NOT open a second dialog).
//
// Run via:   npm run test:unit   (gui/app)
import { test } from "node:test";
import assert from "node:assert/strict";

import { pickFilesViaFileSystemAccess } from "./file-ingest.js";

// ── Minimal shims (Node has no DOM) ───────────────────────────────────────────────────────────

/** The only `File` surfaces the picker path touches are `name` and identity. */
function fakeFile(name: string): File {
  return { name } as unknown as File;
}

/** Install a `window.showOpenFilePicker` stub for one test and restore afterwards. */
async function withPicker<T>(
  picker: unknown,
  body: (calls: unknown[]) => Promise<T>,
): Promise<T> {
  const calls: unknown[] = [];
  const g = globalThis as { window?: unknown };
  const hadWindow = "window" in g;
  const prev = g.window;
  g.window =
    typeof picker === "function"
      ? {
          showOpenFilePicker: (opts?: unknown) => {
            calls.push(opts);
            return (picker as (o?: unknown) => unknown)(opts);
          },
        }
      : {};
  try {
    return await body(calls);
  } finally {
    if (hadWindow) g.window = prev;
    else delete g.window;
  }
}

const handleFor = (file: File) => ({ name: file.name, getFile: async () => file });

// ── The unavailable path: null, so the caller falls back to its own <input type=file> ─────────

test("resolves null when the browser has no showOpenFilePicker", async () => {
  await withPicker(undefined, async () => {
    assert.equal(await pickFilesViaFileSystemAccess(), null);
  });
});

test("resolves null outside a browser (SSR/prerender) instead of throwing", async () => {
  assert.equal(await pickFilesViaFileSystemAccess(), null);
});

test("resolves null when the picker fails for a reason other than a user cancel", async () => {
  await withPicker(
    () => Promise.reject(new DOMException("cross-origin frame", "SecurityError")),
    async () => {
      assert.equal(await pickFilesViaFileSystemAccess(), null);
    },
  );
});

// ── The available path: the RAW File objects, undecoded ───────────────────────────────────────

test("returns the picked File objects themselves, not their text", async () => {
  const a = fakeFile("graph.ttl");
  const b = fakeFile("dump.nt.gz");
  await withPicker(
    async () => [handleFor(a), handleFor(b)],
    async () => {
      const picked = await pickFilesViaFileSystemAccess({ accept: [".ttl", ".gz"] });
      assert.ok(picked, "picker was available, so the result must not be null");
      // Reference identity (assert.equal, not deepEqual): a decompressing caller needs the
      // original binary `File`, not a name-only summary of it.
      assert.equal(picked.files.length, 2);
      assert.equal(picked.files[0], a);
      assert.equal(picked.files[1], b);
      assert.deepEqual(picked.rejected, []);
    },
  );
});

test("forwards multiple + the accept list to the picker as a dialog filter", async () => {
  await withPicker(
    async () => [],
    async (calls) => {
      await pickFilesViaFileSystemAccess({
        accept: [".ttl", ".gz"],
        multiple: false,
        description: "RDF files",
      });
      assert.deepEqual(calls, [
        {
          multiple: false,
          excludeAcceptAllOption: false,
          types: [{ description: "RDF files", accept: { "text/plain": [".ttl", ".gz"] } }],
        },
      ]);
    },
  );
});

// ── A user cancel is an EMPTY result, never null (the caller must not re-prompt) ──────────────

test("resolves an empty result — not null — when the user cancels the dialog", async () => {
  await withPicker(
    () => Promise.reject(new DOMException("The user aborted a request.", "AbortError")),
    async () => {
      assert.deepEqual(await pickFilesViaFileSystemAccess(), { files: [], rejected: [] });
    },
  );
});

// ── No silent drops: an unreadable handle becomes a reason, the rest still come back ──────────

test("rejects an unreadable handle with a reason and keeps the readable files", async () => {
  const ok = fakeFile("good.ttl");
  await withPicker(
    async () => [
      { name: "gone.ttl", getFile: async () => { throw new Error("file was deleted"); } },
      handleFor(ok),
    ],
    async () => {
      const picked = await pickFilesViaFileSystemAccess();
      assert.ok(picked);
      assert.deepEqual(picked.files, [ok]);
      assert.equal(picked.rejected.length, 1);
      assert.equal(picked.rejected[0].name, "gone.ttl");
      assert.match(picked.rejected[0].reason, /file was deleted/);
    },
  );
});
