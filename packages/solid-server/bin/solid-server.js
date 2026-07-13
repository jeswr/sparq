#!/usr/bin/env node
// [GPT-5.6] sq-6xasp.9: npx entry for the local fixed-owner wasm Solid server.
import { startSolidServer } from '../src/index.js';
import { parseArgs, usage } from '../src/cli.js';

async function main() {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    process.stdout.write(usage());
    return;
  }

  const server = await startSolidServer(parsed.options);
  const address = server.address();
  if (!address || typeof address === 'string') {
    throw new Error('listener did not expose a TCP address');
  }
  process.stdout.write(`${JSON.stringify({
    event: 'listening',
    url: `http://127.0.0.1:${address.port}`,
    baseUrl: parsed.options.baseUrl ?? `http://127.0.0.1:${address.port}`,
    ownerWebid: parsed.options.ownerWebid ?? 'https://example.invalid/profile/card#me',
    auth: 'fixed-owner-local-only',
  })}\n`);

  const shutdown = () => {
    void server.closeAsync().catch((error) => {
      process.stderr.write(`${error.message}\n`);
      process.exitCode = 1;
    });
  };
  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);
}

main().catch((error) => {
  process.stderr.write(`solid-server: ${error.message}\n`);
  process.exitCode = 1;
});
