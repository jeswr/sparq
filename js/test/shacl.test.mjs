// [OPUS-4.8] sq-pxls (#162): end-to-end exercise of `SparqStore.validate` —
// the ergonomic SHACL surface PSS's ADR-0014 `ShaclValidator` seam consumes
// (a drop-in for `rdf-validate-shacl`). This runs in a REAL wasm runtime
// against the built `dist/`, so it also proves the SHIPPED bundle was built
// with `--features shacl` (a shacl-less bundle would make `validate` throw the
// "requires a SHACL-enabled wasm bundle" guard).
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { SparqStore } from '../dist/index.js';

const SHAPES = `
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix ex: <http://example.org/> .
  @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
  ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [
      sh:path ex:age ;
      sh:datatype xsd:integer ;
      sh:minInclusive 0 ;
      sh:message "age must be a non-negative integer" ;
    ] ;
    sh:property [
      sh:path ex:name ;
      sh:minCount 1 ;
    ] .
`;

// validate is stateless — it does not consult the receiver's triples — so an
// empty store is the natural caller (mirroring rdf-validate-shacl's API shape).
const emptyStore = () => SparqStore.fromString('', 'turtle');

test('validate() reports conforms:true for conforming data', async () => {
  const store = await emptyStore();
  const report = store.validate(
    `@prefix ex: <http://example.org/> .
     ex:alice a ex:Person ; ex:age 30 ; ex:name "Alice" .`,
    SHAPES,
    'turtle',
  );
  assert.equal(report.conforms, true);
  assert.deepEqual(report.results, []);
});

test('validate() surfaces a violation with focusNode + message for violating data', async () => {
  const store = await emptyStore();
  const report = store.validate(
    `@prefix ex: <http://example.org/> .
     ex:bob a ex:Person ; ex:age -1 .`,
    SHAPES,
    'turtle',
  );
  assert.equal(report.conforms, false);
  assert.ok(report.results.length >= 1, 'at least one violation');

  // The negative-age violation carries the offending focus node, the property
  // path, the declared message, the value, severity and the constraint IRI.
  const ageViolation = report.results.find(
    (r) => r.sourceConstraintComponent.endsWith('MinInclusiveConstraintComponent'),
  );
  assert.ok(ageViolation, 'minInclusive violation present');
  assert.equal(ageViolation.focusNode, '<http://example.org/bob>');
  assert.equal(ageViolation.path, '<http://example.org/age>');
  assert.equal(ageViolation.message, 'age must be a non-negative integer');
  assert.equal(ageViolation.severity, 'http://www.w3.org/ns/shacl#Violation');
  assert.match(ageViolation.value, /-1/);

  // The missing-name (minCount) violation is also reported.
  assert.ok(
    report.results.some((r) => r.sourceConstraintComponent.endsWith('MinCountConstraintComponent')),
    'minCount violation present',
  );
});

test('validate() defaults format to turtle', async () => {
  const store = await emptyStore();
  const report = store.validate(
    `@prefix ex: <http://example.org/> .
     ex:carol a ex:Person ; ex:age 40 ; ex:name "Carol" .`,
    SHAPES,
  );
  assert.equal(report.conforms, true);
});
