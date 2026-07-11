// [FABLE-5] sq-hmd7l.17 — static file server shared by the harness (run.mjs)
// and the cross-LIBRARY comparison (compare.mjs). Extracted verbatim from
// run.mjs (sq-3ul2n.1) so the comparison layers on the SAME harness plumbing
// instead of duplicating it; behaviour is unchanged for run.mjs.

import http from "node:http";
import path from "node:path";
import { readFile } from "node:fs/promises";

export const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm", // required for instantiateStreaming
  ".json": "application/json",
  ".map": "application/json",
};

/**
 * Starts a loopback static server over the given `{ urlPrefix: absoluteDir }`
 * roots. Responses carry explicit `content-length` (chunked transfer would
 * tax the fetch phase unevenly per engine) and `cache-control: no-store`
 * (fetch timings must hit the server, not a disk cache).
 *
 * @param {Record<string, string>} roots
 * @returns {Promise<{ server: import("node:http").Server, port: number }>}
 */
export function startServer(roots) {
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url, "http://localhost");
      const prefix = Object.keys(roots).find((p) => url.pathname.startsWith(p));
      if (!prefix) {
        res.writeHead(404).end("not found");
        return;
      }
      const root = roots[prefix];
      const rel = url.pathname.slice(prefix.length);
      const file = path.resolve(root, rel);
      if (!file.startsWith(root + path.sep) && file !== root) {
        res.writeHead(403).end("forbidden");
        return;
      }
      const body = await readFile(file);
      res.writeHead(200, {
        "content-type": MIME[path.extname(file)] ?? "application/octet-stream",
        "content-length": body.length,
        "cache-control": "no-store",
      });
      res.end(body);
    } catch {
      res.writeHead(404).end("not found");
    }
  });
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => resolve({ server, port: server.address().port }));
  });
}
