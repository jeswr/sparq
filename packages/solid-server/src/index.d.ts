// [GPT-5.6] sq-6xasp.9: public Node host types for the Solid-server wasm package.
import type { Server } from 'node:http';

export interface SolidServerOptions {
  port?: number;
  baseUrl?: string;
  ownerWebid?: string;
}

export interface SolidHttpServer extends Server {
  closeAsync(): Promise<void>;
}

/** Start a loopback-only, fixed-owner, in-memory local-development server. */
export function startSolidServer(options?: SolidServerOptions): Promise<SolidHttpServer>;
