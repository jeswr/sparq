// [OPUS-4.8]
// Hand-written TypeScript declarations for @rdfjs-test/conformance, typed
// against @rdfjs/types. The runtime is authored in plain ESM (src/index.mjs)
// so a `node --test` consumer needs zero build step; these declarations give
// `tsc --noEmit` and downstream consumers full types.

import type {
  DataFactory,
  DatasetCore,
  Quad,
  Source,
  Sink,
  Stream,
} from '@rdfjs/types';

/**
 * A pluggable test API. Defaults to `node:test`'s `describe`/`test` when omitted.
 * Typed loosely (`unknown` args) so any compatible runner — including the
 * `{ skip }`-bearing `node:test` signature — satisfies it.
 */
export interface TestApi {
  describe: (name: string, fn: () => void) => unknown;
  test: (...args: unknown[]) => unknown;
}

/** A DatasetFactory-style builder: `(quads?) => DatasetCore`. */
export type DatasetBuilder = (quads?: Iterable<Quad>) => DatasetCore;

export interface DataFactoryTestOptions extends Partial<TestApi> {
  factory: DataFactory;
  label?: string;
}

export interface DatasetTestOptions extends Partial<TestApi> {
  factory: DataFactory;
  datasetFactory: DatasetBuilder;
  label?: string;
}

export interface StreamTestOptions extends Partial<TestApi> {
  source?: Source;
  sink?: Sink<Stream<Quad>, Stream<Quad>>;
  factory?: DataFactory;
  label?: string;
}

export interface RunAllOptions extends Partial<TestApi> {
  factory?: DataFactory;
  datasetFactory?: DatasetBuilder;
  source?: Source;
  sink?: Sink<Stream<Quad>, Stream<Quad>>;
  label?: string;
}

/**
 * Register the RDF/JS DataFactory + Term-hierarchy conformance sub-suites
 * against `options.factory`.
 */
export function runDataFactoryTests(options: DataFactoryTestOptions): Promise<void>;

/**
 * Register the RDF/JS DatasetCore conformance sub-suite, plus the Dataset
 * algebra sub-suite for whichever algebra methods the dataset implements
 * (feature-detected; a DatasetCore-only impl still passes the core part).
 */
export function runDatasetTests(options: DatasetTestOptions): Promise<void>;

/**
 * Register the OPTIONAL RDF/JS Stream/Source/Sink conformance sub-suite. A
 * no-op-with-skip when neither `source` nor `sink` is provided.
 */
export function runStreamTests(options: StreamTestOptions): Promise<void>;

/** Run whichever suites are wired in `impl`. */
export function runAll(impl: RunAllOptions): Promise<void>;

declare const _default: {
  runDataFactoryTests: typeof runDataFactoryTests;
  runDatasetTests: typeof runDatasetTests;
  runStreamTests: typeof runStreamTests;
  runAll: typeof runAll;
};
export default _default;
