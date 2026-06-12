export { SparqStore, type RdfFormat, type SparqStoreOptions } from './store.js';
export { Bindings } from './bindings.js';
export { DataFactory, NamedNode, BlankNode, Literal, Variable, DefaultGraph, Quad } from './terms.js';
export {
  termFromSparqlJson,
  termToNT,
  quadsToNQuads,
  detectQueryForm,
  askToSelect,
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
