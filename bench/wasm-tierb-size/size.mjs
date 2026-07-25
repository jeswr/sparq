#!/usr/bin/env node
// [GPT-5.6] sq-3k4rp — deterministic shipped Tier-B WASM size envelope.

import path from "node:path";
import { createHash } from "node:crypto";
import { gzipSync, constants as zlibConstants } from "node:zlib";
import { spawnSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import process from "node:process";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..", "..");
const OUT = path.resolve(process.argv[2] ?? path.join(HERE, "envelope.json"));

const BUNDLES = [
  ["sparq-rsp-wasm", "sparq_rsp_wasm_bg.wasm"],
  ["sparq-text-wasm", "sparq_text_wasm_bg.wasm"],
  ["sparq-reason-wasm", "sparq_reason_wasm_bg.wasm"],
  ["sparq-shacl-wasm", "sparq_shacl_wasm_bg.wasm"],
];

function commandVersion(command, args = ["--version"]) {
  const result = spawnSync(command, args, { encoding: "utf8" });
  if (result.status !== 0) return null;
  return result.stdout.trim() || result.stderr.trim() || null;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function measure(crate, filename) {
  const relativePath = path.join("crates", crate, "pkg", filename);
  try {
    const bytes = await readFile(path.join(REPO, relativePath));
    return {
      crate,
      artifact: relativePath,
      status: "present",
      raw_bytes: bytes.length,
      gzip_level_9_bytes_advisory: gzipSync(bytes, {
        level: 9,
        // Pin fields that can otherwise vary between gzip producers.
        mtime: 0,
      }).length,
      sha256: sha256(bytes),
    };
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    console.error(`[wasm-tierb-size] notice: ${relativePath} is missing; build ${crate} first`);
    return {
      crate,
      artifact: relativePath,
      status: "missing",
      notice: "artifact was not built; no size or digest was recorded",
    };
  }
}

const git = spawnSync("git", ["-C", REPO, "rev-parse", "HEAD"], { encoding: "utf8" });
const envelope = {
  schema_version: 1,
  suite: "wasm-tierb-size",
  bead: "sq-3k4rp",
  deterministic_metric: "raw_bytes",
  git_commit: git.status === 0 ? git.stdout.trim() : null,
  toolchain: {
    wasm_pack: commandVersion("wasm-pack"),
    binaryen: commandVersion("wasm-opt"),
    node_zlib: process.versions.zlib,
    gzip_level: zlibConstants.Z_BEST_COMPRESSION,
  },
  bundles: await Promise.all(BUNDLES.map(([crate, filename]) => measure(crate, filename))),
};

await writeFile(OUT, `${JSON.stringify(envelope, null, 2)}\n`);

for (const bundle of envelope.bundles) {
  if (bundle.status === "present") {
    console.log(
      `${bundle.crate}: ${bundle.raw_bytes} raw bytes; ` +
        `${bundle.gzip_level_9_bytes_advisory} gzip-9 advisory bytes; sha256 ${bundle.sha256}`,
    );
  } else {
    console.log(`${bundle.crate}: missing-with-notice`);
  }
}
console.log(`wrote ${path.relative(process.cwd(), OUT)}`);
