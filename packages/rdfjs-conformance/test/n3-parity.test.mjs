// [OPUS-4.8]
// @ts-check
// Parity proof: the conformance harness is implementation-agnostic.
// It runs the SAME suites against two real, third-party RDF/JS libraries —
// N3.js (DataFactory + Store) and @rdfjs/dataset — and they pass.
//
//   • N3.js provides a full DataFactory AND an N3.Store that is a DatasetCore
//     implementing the full Dataset algebra.
//   • @rdfjs/dataset provides ONLY a DatasetFactory (`dataset(quads?)` → a
//     DatasetCore); it ships NO DataFactory and NO Dataset algebra. So we drive
//     its DatasetCore part with N3's DataFactory as the (implementation-agnostic)
//     quad builder, and its algebra methods are feature-detect-skipped — which
//     is exactly correct, and demonstrates the harness adapts to a DatasetCore-
//     only impl.
//
// `node --test` over this file is GREEN, proving the harness runs cleanly
// against genuine third-party implementations.

import N3 from 'n3';
import datasetMod from '@rdfjs/dataset';
import { runDataFactoryTests, runDatasetTests } from '../src/index.mjs';

const { DataFactory, Store } = N3;

// ── N3.js: DataFactory conformance ──────────────────────────────────────────
await runDataFactoryTests({ factory: DataFactory, label: 'n3' });

// ── N3.js: full Dataset (Store is a DatasetCore + implements the algebra) ────
await runDatasetTests({
  factory: DataFactory,
  datasetFactory: (quads) => new Store(quads ? [...quads] : undefined),
  label: 'n3 (Store)',
});

// ── @rdfjs/dataset: DatasetCore conformance (algebra feature-detect-skipped) ─
// @rdfjs/dataset has no DataFactory of its own, so we build quads with N3's
// DataFactory — the suites compare quads implementation-agnostically, so a
// foreign-built quad is a first-class member of an @rdfjs/dataset DatasetCore.
await runDatasetTests({
  factory: DataFactory,
  datasetFactory: (quads) => datasetMod.dataset(quads ? [...quads] : undefined),
  label: '@rdfjs/dataset',
});
