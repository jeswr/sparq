export {
  SparqStore,
  type RdfFormat,
  type SparqStoreOptions,
  type SerializeFormat,
  type SerializeOptions,
  type ValidationReport,
  type ValidationResult,
} from './store.js';
// [OPUS-4.8] sq-lii76 (#981) — the RDF/JS `DatasetCore` entry the ESM `<script type=module>`
// snippet imports by name (`import { Dataset } from "..."`), lazily instantiating the wasm.
export { Dataset } from './dataset.js';
export { Bindings } from './bindings.js';
export { DataFactory, NamedNode, BlankNode, Literal, Variable, DefaultGraph, Quad } from './terms.js';
export {
  termFromSparqlJson,
  termToNT,
  quadsToNQuads,
  parseNTriples,
  detectQueryForm,
  SparqlJsonRowsParser,
  type SparqlJsonTerm,
  type SparqlJsonResults,
  type QueryForm,
} from './sparql.js';
export { init } from './wasm.js';
export { decompress, decompressToString, sniffCodec, type CompressionCodec } from './decompress.js';
export {
  SparqDictionaryClient,
  dictIdOf,
  verifyDictId,
  parseZstdDictId,
  SPARQ_DICTIONARY_HEADER,
  SPARQ_DICTIONARY_CURRENT_HEADER,
  type DictionaryDecoder,
  type DictionaryFetchResult,
  type SparqDictionaryClientOptions,
} from './dictionary.js';
