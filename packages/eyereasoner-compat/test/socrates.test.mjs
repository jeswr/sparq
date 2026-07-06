// [FABLE-5] sq-ohnj1 — the eye-js README socrates example, run UNMODIFIED against
// @sparq-org/eyereasoner-compat (the issue's acceptance sketch). Verbatim from
// https://github.com/eyereasoner/eye-js README.
import test from 'node:test';
import assert from 'node:assert/strict';
import { n3reasoner } from '../dist/index.js';

const queryString = `
@prefix : <http://example.org/socrates#>.

{:Socrates a ?WHAT} => {:Socrates a ?WHAT}.
`;

const dataString = `
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#>.
@prefix : <http://example.org/socrates#>.

:Socrates a :Human.
:Human rdfs:subClassOf :Mortal.

{?A rdfs:subClassOf ?B. ?S a ?A} => {?S a ?B}.
`;

const MORTAL = 'http://example.org/socrates#Mortal';
const HUMAN = 'http://example.org/socrates#Human';

test('socrates: n3reasoner(data, query) returns a string with the entailed Mortal typing', async () => {
  // The result of the query (as a string) — the eye-js README idiom, unmodified.
  const resultString = await n3reasoner(dataString, queryString);
  assert.equal(typeof resultString, 'string');
  // The query `{:Socrates a ?WHAT} => {:Socrates a ?WHAT}` selects every `:Socrates a ?WHAT`
  // over the closure — including the ENTAILED `:Socrates a :Mortal`.
  assert.ok(resultString.includes(MORTAL), `expected Mortal typing in:\n${resultString}`);
  assert.ok(resultString.includes(HUMAN), `expected asserted Human typing in:\n${resultString}`);
});

test('socrates: n3reasoner(data) returns all inferred data (derivations default)', async () => {
  // All inferred data — the eye-js README idiom, unmodified.
  const resultString = await n3reasoner(dataString);
  assert.equal(typeof resultString, 'string');
  // Default output is `derivations` (EYE --pass-only-new): only the newly-derived Mortal typing.
  assert.ok(resultString.includes(MORTAL), `expected derived Mortal typing in:\n${resultString}`);
});
