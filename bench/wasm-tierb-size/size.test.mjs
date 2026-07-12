// [GPT-5.6] sq-3k4rp — mutation witness for present and absent bundle accounting.

import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..", "..");
const artifact = path.join(REPO, "crates", "sparq-rsp-wasm", "pkg", "sparq_rsp_wasm_bg.wasm");
const tmp = await mkdtemp(path.join(os.tmpdir(), "wasm-tierb-size-test-"));
const out = path.join(tmp, "envelope.json");

try {
  await mkdir(path.dirname(artifact), { recursive: true });
  const original = await readFile(artifact).catch(() => null);
  try {
    await writeFile(artifact, Buffer.from("deterministic-fixture"));
    const run = spawnSync(process.execPath, [path.join(HERE, "size.mjs"), out], { encoding: "utf8" });
    assert.equal(run.status, 0, run.stderr);
    const envelope = JSON.parse(await readFile(out, "utf8"));
    assert.equal(envelope.bundles.length, 4);
    const present = envelope.bundles.find((row) => row.crate === "sparq-rsp-wasm");
    assert.deepEqual(
      { status: present.status, raw_bytes: present.raw_bytes, sha256: present.sha256 },
      {
        status: "present",
        raw_bytes: 21,
        sha256: "d05270b97c3ebfff4627da61d16ef5a480fab6f967cefebcac14b079aa01be10",
      },
    );
    assert.ok(envelope.bundles.some((row) => row.status === "missing"));
  } finally {
    if (original === null) await rm(artifact, { force: true });
    else await writeFile(artifact, original);
  }
} finally {
  await rm(tmp, { recursive: true, force: true });
}
