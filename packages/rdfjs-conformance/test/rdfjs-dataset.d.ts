// [OPUS-4.8]
// Ambient declaration for `@rdfjs/dataset`, which ships no TypeScript types of
// its own. Scoped to what the parity test consumes: an RDF/JS DataFactory whose
// default export also exposes a `.dataset(quads?)` DatasetCore builder.
declare module '@rdfjs/dataset' {
  import type { DataFactory, DatasetCore, Quad } from '@rdfjs/types';
  interface RdfjsDatasetModule extends DataFactory {
    dataset(quads?: Iterable<Quad>): DatasetCore;
  }
  const mod: RdfjsDatasetModule;
  export default mod;
}
