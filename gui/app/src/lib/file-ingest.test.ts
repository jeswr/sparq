// [SONNET-4.6] sq-3uzlf — regression coverage for consumer-specific file readers.
import { test } from "node:test";
import assert from "node:assert/strict";

import { readDroppedFiles } from "./file-ingest.js";

interface FakeFileOptions {
  name: string;
  text?: () => Promise<string>;
}

function fakeFile({ name, text = async () => "default text" }: FakeFileOptions): File {
  return { name, size: 12, text } as File;
}

function droppedFile(file: File): DataTransfer {
  return {
    items: [],
    files: [file],
  } as unknown as DataTransfer;
}

test("[SONNET-4.6] sq-3uzlf: readDroppedFiles uses the configured reader", async () => {
  let defaultReaderCalled = false;
  let customReaderCalled = false;
  const file = fakeFile({
    name: "dataset.ttl",
    text: async () => {
      defaultReaderCalled = true;
      return "wrong";
    },
  });

  const result = await readDroppedFiles(droppedFile(file), {
    readFile: async (received) => {
      customReaderCalled = true;
      assert.strictEqual(received, file);
      return "decoded text";
    },
  });

  assert.strictEqual(customReaderCalled, true);
  assert.strictEqual(defaultReaderCalled, false);
  assert.deepStrictEqual(result, {
    accepted: [{ name: "dataset.ttl", text: "decoded text", bytes: 12 }],
    rejected: [],
  });
});

test("[SONNET-4.6] sq-3uzlf: reader failures become rejected entries", async () => {
  const file = fakeFile({ name: "broken.ttl" });
  const result = await readDroppedFiles(droppedFile(file), {
    readFile: async () => {
      throw new Error("decoder failed");
    },
  });

  assert.deepStrictEqual(result, {
    accepted: [],
    rejected: [{ name: "broken.ttl", reason: "could not be read: decoder failed" }],
  });
});

test("[SONNET-4.6] sq-3uzlf: rejected extensions do not invoke the reader", async () => {
  let readerCalled = false;
  const file = fakeFile({ name: "dataset.csv" });
  const result = await readDroppedFiles(droppedFile(file), {
    accept: [".ttl"],
    readFile: async () => {
      readerCalled = true;
      return "wrong";
    },
  });

  assert.strictEqual(readerCalled, false);
  assert.strictEqual(result.accepted.length, 0);
  assert.strictEqual(result.rejected.length, 1);
  assert.strictEqual(result.rejected[0]?.name, "dataset.csv");
  assert.match(result.rejected[0]?.reason ?? "", /unsupported file type/);
});
