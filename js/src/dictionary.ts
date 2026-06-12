/**
 * Client side of sparq's dictionary-fetch protocol
 * (research/custom-parsers-D4-compressed-serialization.md §4): servers
 * compress SMALL responses with a shared zstd *vocabulary dictionary* (5×
 * smaller on ≤1 KiB bodies), but only when the client PROVES it already holds
 * that dictionary — so the first request is plain zstd and no request ever
 * waits on a dictionary round-trip.
 *
 * Wire contract:
 * - request `Sparq-Dictionary: <dict-id>[, <dict-id>…]` — the ids this client
 *   holds (a dictionary is immutable and content-addressed:
 *   `dict-id = base64url(truncated SHA-256 of its bytes)`).
 * - response `Sparq-Dictionary: <dict-id>` — the dictionary the body was
 *   compressed with (absent ⇒ plain zstd frames / platform content-encoding).
 * - response `Sparq-Dictionary-Current: <dict-id>` — the newest dictionary;
 *   the client warms it up (`GET <origin>/dictionary/<dict-id>`, immutable,
 *   infinitely cacheable) in the background for the *next* request.
 *
 * fzstd (this package's bundled decoder) cannot decode dictionary frames, so
 * dictionary-compressed bodies need a dict-capable decoder via
 * `decodeWithDictionary` (e.g. zstd-wasm, or `node:zlib`'s
 * `zstdDecompressSync(body, { dictionary })` in Node). Without the hook the
 * client still works — it simply never advertises dictionaries, keeping every
 * response plain.
 */
import { decompress, sniffCodec } from './decompress.js';

export const SPARQ_DICTIONARY_HEADER = 'Sparq-Dictionary';
export const SPARQ_DICTIONARY_CURRENT_HEADER = 'Sparq-Dictionary-Current';

/** Decodes a zstd body compressed with `dictionary` (e.g. backed by zstd-wasm). */
export type DictionaryDecoder = (body: Uint8Array, dictionary: Uint8Array) => Uint8Array | Promise<Uint8Array>;

export interface SparqDictionaryClientOptions {
  /** The `fetch` implementation to wrap (defaults to the global one). */
  fetch?: typeof fetch;
  /**
   * Dict-capable zstd decoder. REQUIRED for the dictionary path to engage:
   * without it the client never advertises held dictionaries (and so never
   * receives a dictionary-compressed body).
   */
  decodeWithDictionary?: DictionaryDecoder;
  /** Resolves the dictionary-fetch URL. Default: `<origin>/dictionary/<dict-id>`. */
  dictionaryUrl?: (origin: string, dictId: string) => string;
}

const b64url = (bytes: Uint8Array): string => {
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
};

/**
 * The content address of a dictionary: base64url(SHA-256). Servers truncate
 * it; [`verifyDictId`] therefore checks by PREFIX.
 */
export async function dictIdOf(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', bytes as BufferSource);
  return b64url(new Uint8Array(digest));
}

/** Whether `claimed` (a possibly truncated dict-id) addresses `bytes`. */
export async function verifyDictId(bytes: Uint8Array, claimed: string): Promise<boolean> {
  return claimed.length > 0 && (await dictIdOf(bytes)).startsWith(claimed);
}

/**
 * The 32-bit `Dictionary_ID` a zstd frame header names (RFC 8878 §3.1.1), so
 * a client can detect that a frame needs a (trained) dictionary before asking
 * the server which: `0` = the frame names none (plain, or a raw-content
 * dictionary); `null` = not a zstd frame. The Sparq-Dictionary response
 * header remains the protocol's authoritative signal.
 */
export function parseZstdDictId(frame: Uint8Array): number | null {
  if (frame.length < 5 || !(frame[0] === 0x28 && frame[1] === 0xb5 && frame[2] === 0x2f && frame[3] === 0xfd)) {
    return null;
  }
  const fhd = frame[4]!;
  const didSize = [0, 1, 2, 4][fhd & 0x03]!;
  const singleSegment = (fhd & 0x20) !== 0;
  let offset = 5 + (singleSegment ? 0 : 1); // Window_Descriptor byte unless single-segment
  if (didSize === 0) return 0;
  if (frame.length < offset + didSize) return null;
  let id = 0;
  for (let i = didSize - 1; i >= 0; i--) id = id * 256 + frame[offset + i]!;
  return id;
}

/** What [`SparqDictionaryClient.fetch`] resolves to. */
export interface DictionaryFetchResult {
  /** The decoded (decompressed) body bytes. */
  body: Uint8Array;
  /** The dict-id the body was compressed with, if the dictionary path engaged. */
  dictionary?: string;
  /** The underlying `Response` (headers/status; its body is consumed). */
  response: Response;
}

/**
 * A `fetch` wrapper implementing the client side of the dictionary-fetch
 * protocol: advertises held dictionaries, decodes dictionary-compressed
 * bodies via the configured hook, decodes plain zstd/gzip bodies via the
 * bundled codecs (when the platform did not already handle the
 * content-encoding), and warms up newly advertised dictionaries in the
 * background — content-verified against their id before being trusted.
 */
export class SparqDictionaryClient {
  #fetch: typeof fetch;
  #decodeWithDictionary: DictionaryDecoder | undefined;
  #dictionaryUrl: (origin: string, dictId: string) => string;
  #dictionaries = new Map<string, Uint8Array>();
  #warmups = new Map<string, Promise<void>>();

  constructor(options: SparqDictionaryClientOptions = {}) {
    this.#fetch = options.fetch ?? fetch;
    this.#decodeWithDictionary = options.decodeWithDictionary;
    this.#dictionaryUrl = options.dictionaryUrl ?? ((origin, id) => `${origin}/dictionary/${encodeURIComponent(id)}`);
  }

  /** The dict-ids this client currently holds (advertised on each request). */
  get dictionaryIds(): string[] {
    return [...this.#dictionaries.keys()];
  }

  /**
   * Seeds a dictionary (e.g. persisted from a previous session). With `id`
   * (the server's possibly truncated form) the bytes are verified against it;
   * without, the full content address is computed. Returns the id used.
   */
  async addDictionary(bytes: Uint8Array, id?: string): Promise<string> {
    if (id !== undefined && !(await verifyDictId(bytes, id))) {
      throw new Error(`dictionary bytes do not hash to the claimed dict-id ${JSON.stringify(id)}`);
    }
    const key = id ?? (await dictIdOf(bytes));
    this.#dictionaries.set(key, bytes);
    return key;
  }

  /**
   * Performs the request with dictionary negotiation. Dictionary-compressed
   * and plain-zstd/gzip bodies come back decoded; anything else passes
   * through untouched (e.g. when the platform already decoded the
   * content-encoding).
   */
  async fetch(url: string | URL, init?: RequestInit): Promise<DictionaryFetchResult> {
    const headers = new Headers(init?.headers);
    // Only advertise what we can actually decode: without a dict-capable
    // decoder the server must keep sending plain frames.
    if (this.#decodeWithDictionary) {
      for (const id of this.#dictionaries.keys()) headers.append(SPARQ_DICTIONARY_HEADER, id);
    }
    const response = await this.#fetch(url, { ...init, headers });
    const raw = new Uint8Array(await response.arrayBuffer());

    // Warm up the advertised newest dictionary for the NEXT request.
    const current = response.headers.get(SPARQ_DICTIONARY_CURRENT_HEADER);
    if (current) this.#warmup(new URL(response.url || url).origin, current);

    const used = response.headers.get(SPARQ_DICTIONARY_HEADER);
    if (used) {
      const dictionary = this.#dictionaries.get(used);
      if (!dictionary) {
        // Protocol violation (we never advertised it) — fail loudly rather
        // than return bytes the caller cannot decode.
        throw new Error(`server compressed with unheld dictionary ${JSON.stringify(used)}`);
      }
      const decode = this.#decodeWithDictionary;
      if (!decode) {
        throw new Error(
          'dictionary-compressed response but no decodeWithDictionary hook configured ' +
            '(fzstd cannot decode dictionary frames — supply e.g. a zstd-wasm-backed decoder)',
        );
      }
      return { body: await decode(raw, dictionary), dictionary: used, response };
    }
    // Plain body: decode zstd/gzip if the payload still carries codec magic
    // (i.e. the platform did not already decode a Content-Encoding).
    return { body: sniffCodec(raw) ? await decompress(raw) : raw, response };
  }

  /** Resolves when every in-flight dictionary warm-up has settled. */
  async idle(): Promise<void> {
    while (this.#warmups.size > 0) await Promise.allSettled([...this.#warmups.values()]);
  }

  /** Starts (at most one) background fetch of dictionary `id`. */
  #warmup(origin: string, id: string): void {
    if (this.#dictionaries.has(id) || this.#warmups.has(id)) return;
    const task = (async () => {
      const res = await this.#fetch(this.#dictionaryUrl(origin, id));
      if (!res.ok) throw new Error(`dictionary fetch failed: HTTP ${res.status}`);
      const bytes = new Uint8Array(await res.arrayBuffer());
      // Content-addressing is the integrity model: never trust unverified bytes.
      if (!(await verifyDictId(bytes, id))) {
        throw new Error(`dictionary ${JSON.stringify(id)} failed content verification`);
      }
      this.#dictionaries.set(id, bytes);
    })();
    // Settle-and-forget: a failed warm-up only costs ratio on later requests;
    // clearing the slot lets a future advertisement retry.
    this.#warmups.set(
      id,
      task.catch(() => {}).finally(() => this.#warmups.delete(id)),
    );
  }
}
