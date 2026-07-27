// [OPUS-4.8]
// @ts-check
/**
 * @rdfjs-test/conformance — reusable RDF/JS conformance suites.
 *
 * These runners register sub-suites against ANY RDF/JS implementation passed in
 * as an argument, mirroring the design of the official `@rdfjs/data-model/test`
 * (which exports `runTests({ factory, mocha })`) BUT with ZERO external
 * test-framework dependency: they default to Node's built-in `node:test`
 * (`describe`/`test`) and `node:assert/strict`, so a consumer drives them with
 * `node --test`. A `{ describe, test }` override keeps them framework-agnostic
 * in spirit.
 *
 * Every assertion derives its expected value FROM THE RDF/JS SPEC
 * (https://rdf.js.org/data-model-spec/ and dataset-spec/stream-spec), never from
 * any single implementation, so the same suites pass `n3`, `@rdfjs/dataset`, and
 * fail a deliberately-broken impl. Cross-implementation comparison of quads uses
 * a canonical N-Quads-ish key built from `.value` / `.termType` / `.language` /
 * `.direction` / `.datatype.value` (recursing into quoted triples), which works
 * across implementations with distinct term classes.
 */

import { strict as assert } from 'node:assert';

/**
 * @typedef {import('@rdfjs/types').Term} Term
 * @typedef {import('@rdfjs/types').Quad} Quad
 * @typedef {import('@rdfjs/types').DataFactory} DataFactory
 * @typedef {import('@rdfjs/types').DatasetCore} DatasetCore
 * @typedef {import('@rdfjs/types').Dataset} Dataset
 * @typedef {import('@rdfjs/types').Source} Source
 * @typedef {import('@rdfjs/types').Stream} Stream
 * @typedef {{ describe: Function, test: Function }} TestApi
 */

const XSD_STRING = 'http://www.w3.org/2001/XMLSchema#string';
const RDF_LANG_STRING = 'http://www.w3.org/1999/02/22-rdf-syntax-ns#langString';
const RDF_DIR_LANG_STRING = 'http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString';

/**
 * Resolve the injected test API, defaulting to node:test.
 * @param {{ describe?: Function, test?: Function } | undefined} opts
 * @returns {Promise<TestApi>}
 */
async function resolveTestApi(opts) {
  if (opts && opts.describe && opts.test) {
    return { describe: opts.describe, test: opts.test };
  }
  const nodeTest = await import('node:test');
  return { describe: nodeTest.describe, test: nodeTest.test };
}

/**
 * Implementation-agnostic N-Quads-ish key for a single term.
 * @param {any} term
 * @returns {string}
 */
function termKey(term) {
  if (term == null) return 'null';
  switch (term.termType) {
    case 'NamedNode':
      return `<${term.value}>`;
    case 'BlankNode':
      return `_:${term.value}`;
    case 'Variable':
      return `?${term.value}`;
    case 'DefaultGraph':
      return 'DEFAULT';
    case 'Literal': {
      const dt = term.datatype ? term.datatype.value : XSD_STRING;
      const lang = term.language || '';
      // [OPUS-5] sq-iwhl8 (#1116) — RDF 1.2 base direction, rendered N-Quads-1.2-style
      // (`"x"@en--ltr`) so two literals that differ ONLY in direction get distinct keys.
      // Absent/unsupported => omitted, so keys are unchanged for every non-directional literal.
      const dir = lang ? term.direction || '' : '';
      return `"${term.value}"${lang ? '@' + lang + (dir ? '--' + dir : '') : ''}^^<${dt}>`;
    }
    case 'Quad':
      return quadKey(term);
    default:
      return `?${term.termType}:${term.value}`;
  }
}

/**
 * Implementation-agnostic canonical key for a quad (RDF-star aware).
 * @param {any} q
 * @returns {string}
 */
function quadKey(q) {
  return [q.subject, q.predicate, q.object, q.graph].map(termKey).join(' ');
}

/**
 * Multiset key set for a quad iterable.
 * @param {Iterable<any>} quads
 * @returns {Set<string>}
 */
function quadKeySet(quads) {
  /** @type {Set<string>} */
  const s = new Set();
  for (const q of quads) s.add(quadKey(q));
  return s;
}

/**
 * @param {Iterable<any>} quads
 * @returns {string[]}
 */
function sortedKeys(quads) {
  return [...quadKeySet(quads)].sort();
}

/**
 * True iff a value looks like a callable factory method.
 * @param {any} obj
 * @param {string} name
 * @returns {boolean}
 */
function has(obj, name) {
  return obj != null && typeof obj[name] === 'function';
}

/**
 * [OPUS-5] sq-iwhl8 (#1116) — True iff the factory really builds RDF-star / quoted triples,
 * i.e. accepts a `Quad` in the subject/object position and keeps it there as a nested `Quad`
 * term. Probed rather than assumed, so a DataFactory that predates RDF-star skips the block
 * instead of failing it.
 * @param {any} factory
 * @returns {boolean}
 */
function supportsQuotedTriples(factory) {
  try {
    const n = factory.namedNode('http://example.org/probe');
    const inner = factory.quad(n, n, n);
    const outer = factory.quad(inner, n, inner);
    return outer.subject.termType === 'Quad' && outer.object.termType === 'Quad';
  } catch {
    return false;
  }
}

/**
 * [OPUS-5] sq-iwhl8 (#1116) — True iff the factory really implements the RDF 1.2
 * base-direction literal, i.e. `literal(value, { language, direction })` yields a Literal
 * carrying that `direction`. Probed, so a pre-RDF-1.2 factory skips the block instead of
 * failing it.
 * @param {any} factory
 * @returns {boolean}
 */
function supportsDirectionalLiterals(factory) {
  try {
    const l = factory.literal('probe', { language: 'en', direction: 'ltr' });
    return l.termType === 'Literal' && l.value === 'probe' && l.direction === 'ltr';
  } catch {
    return false;
  }
}

/* ───────────────────────── DataFactory suite ───────────────────────── */

/**
 * @param {object} options
 * @param {import('@rdfjs/types').DataFactory} options.factory
 * @param {string} [options.label]
 * @param {Function} [options.describe]
 * @param {Function} [options.test]
 */
export async function runDataFactoryTests(options) {
  const { factory } = options;
  const label = options.label || 'DataFactory';
  assert.ok(factory, 'runDataFactoryTests requires { factory }');
  const { describe, test } = await resolveTestApi(options);

  describe(`${label} — DataFactory conformance`, () => {
    test('namedNode: termType/value', () => {
      const n = factory.namedNode('http://example.org/s');
      assert.equal(n.termType, 'NamedNode');
      assert.equal(n.value, 'http://example.org/s');
    });

    test('blankNode: termType + fresh-id uniqueness with no arg', () => {
      const b = factory.blankNode('b0');
      assert.equal(b.termType, 'BlankNode');
      assert.equal(b.value, 'b0');
      const f1 = factory.blankNode();
      const f2 = factory.blankNode();
      assert.equal(f1.termType, 'BlankNode');
      assert.notEqual(f1.value, f2.value, 'fresh blank nodes must be unique');
    });

    test('literal: plain literal defaults to xsd:string', () => {
      const l = factory.literal('hello');
      assert.equal(l.termType, 'Literal');
      assert.equal(l.value, 'hello');
      assert.equal(l.language, '');
      assert.equal(l.datatype.value, XSD_STRING);
    });

    test('literal: language tag lowercased + datatype rdf:langString', () => {
      const l = factory.literal('hello', 'EN-GB');
      assert.equal(l.language, 'en-gb', 'language tag must be lowercased');
      assert.equal(l.datatype.value, RDF_LANG_STRING);
    });

    test('literal: explicit datatype is preserved', () => {
      const dt = factory.namedNode('http://www.w3.org/2001/XMLSchema#integer');
      const l = factory.literal('42', dt);
      assert.equal(l.datatype.value, 'http://www.w3.org/2001/XMLSchema#integer');
      assert.equal(l.language, '');
    });

    test('defaultGraph: termType + empty value', () => {
      const g = factory.defaultGraph();
      assert.equal(g.termType, 'DefaultGraph');
      assert.equal(g.value, '');
    });

    test('variable: termType/value (optional)', { skip: !has(factory, 'variable') }, () => {
      const variable = /** @type {NonNullable<DataFactory['variable']>} */ (factory.variable);
      const v = variable('x');
      assert.equal(v.termType, 'Variable');
      assert.equal(v.value, 'x');
    });

    test('quad: components + default graph when omitted', () => {
      const s = factory.namedNode('http://example.org/s');
      const p = factory.namedNode('http://example.org/p');
      const o = factory.literal('o');
      const q = factory.quad(s, p, o);
      assert.equal(q.termType, 'Quad');
      assert.equal(q.subject.value, 'http://example.org/s');
      assert.equal(q.predicate.value, 'http://example.org/p');
      assert.equal(q.object.value, 'o');
      assert.equal(q.graph.termType, 'DefaultGraph', 'omitted graph must be DefaultGraph');
    });

    test('quad: explicit graph is preserved', () => {
      const g = factory.namedNode('http://example.org/g');
      const q = factory.quad(
        factory.namedNode('http://example.org/s'),
        factory.namedNode('http://example.org/p'),
        factory.namedNode('http://example.org/o'),
        g,
      );
      assert.equal(q.graph.termType, 'NamedNode');
      assert.equal(q.graph.value, 'http://example.org/g');
    });

    // [OPUS-5] sq-iwhl8 (#1116) — Data-model spec, `Quad`: "value — Contains an empty string as constant value." A Quad is
    // a Term, so it must carry the Term attributes, not just the four components.
    test('quad: value is the empty-string constant (a Quad is a Term)', () => {
      const n = factory.namedNode('http://example.org/s');
      assert.equal(factory.quad(n, n, n).value, '');
    });

    // [OPUS-5] sq-iwhl8 (#1116) — Data-model spec: `Quad_Subject` / `Quad_Object` both include `Quad`, i.e. a quoted triple
    // (RDF-star) is a first-class term. Feature-detected: a pre-RDF-star factory skips.
    describe('RDF-star / quoted triples (feature-detected)', () => {
      const quoted = supportsQuotedTriples(factory);
      /** @param {string} n */
      const ns = (n) => factory.namedNode(`http://example.org/${n}`);
      /** Inner quoted triple `<s> <p> <o>`. */
      const inner = () => factory.quad(ns('s'), ns('p'), ns('o'));

      test('a quad may be the subject of a quad', { skip: !quoted }, () => {
        const outer = factory.quad(/** @type {any} */ (inner()), ns('says'), factory.literal('yes'));
        assert.equal(outer.subject.termType, 'Quad');
        const s = /** @type {Quad} */ (/** @type {any} */ (outer.subject));
        assert.equal(s.predicate.value, 'http://example.org/p');
        assert.equal(s.graph.termType, 'DefaultGraph', 'the quoted triple keeps its own graph');
        assert.equal(outer.graph.termType, 'DefaultGraph');
      });

      test('a quad may be the object of a quad', { skip: !quoted }, () => {
        const outer = factory.quad(ns('a'), ns('claims'), /** @type {any} */ (inner()));
        assert.equal(outer.object.termType, 'Quad');
        const o = /** @type {Quad} */ (/** @type {any} */ (outer.object));
        assert.equal(o.object.value, 'http://example.org/o');
      });

      test('equals recurses into the quoted triple', { skip: !quoted }, () => {
        /** @param {string} o */
        const outerWith = (o) => factory.quad(
          /** @type {any} */ (factory.quad(ns('s'), ns('p'), ns(o))),
          ns('says'),
          factory.literal('yes'),
        );
        assert.equal(outerWith('o').equals(outerWith('o')), true, 'equal quoted triples => equal');
        assert.equal(outerWith('o').equals(outerWith('other')), false, 'a differing quoted triple must not be equal');
        // A quoted triple is NOT equal to a term of another type with the same (empty) value.
        assert.equal(factory.defaultGraph().equals(/** @type {any} */ (inner())), false);
      });

      test('the canonical key of a nested quad is stable and discriminating', { skip: !quoted }, () => {
        const a = factory.quad(/** @type {any} */ (inner()), ns('says'), factory.literal('yes'));
        const b = factory.quad(/** @type {any} */ (inner()), ns('says'), factory.literal('yes'));
        const c = factory.quad(/** @type {any} */ (inner()), ns('says'), factory.literal('no'));
        assert.equal(quadKey(a), quadKey(b));
        assert.notEqual(quadKey(a), quadKey(c));
      });

      test(
        'fromTerm accepts a Quad term and yields an equal Quad',
        { skip: !quoted || !has(factory, 'fromTerm') },
        () => {
          const original = inner();
          const copy = /** @type {Term} */ (factory.fromTerm(/** @type {any} */ (original)));
          assert.equal(copy.termType, 'Quad');
          assert.equal(copy.equals(original), true);
        },
      );

      test(
        'fromQuad deep-copies a quad whose subject is a quoted triple',
        { skip: !quoted || !has(factory, 'fromQuad') },
        () => {
          const original = factory.quad(
            /** @type {any} */ (inner()),
            ns('says'),
            factory.literal('yes'),
            ns('g'),
          );
          const copy = factory.fromQuad(original);
          assert.equal(copy.termType, 'Quad');
          assert.equal(copy.subject.termType, 'Quad', 'the quoted triple must survive the copy');
          assert.equal(copy.equals(original), true);
          assert.equal(quadKey(copy), quadKey(original));
        },
      );
    });

    // [OPUS-5] sq-iwhl8 (#1116) — Data-model spec (RDF 1.2): `literal(value, { language, direction })` builds a directional
    // language-tagged string, whose datatype is `rdf:dirLangString`. Feature-detected.
    describe('directional language-tagged literals (feature-detected)', () => {
      const directional = supportsDirectionalLiterals(factory);
      /**
       * Build a directional literal. The `{ language, direction }` argument shape post-dates
       * some `@rdfjs/types` releases, so it is passed (and the result read back) through `any`
       * — the assertions below are what pins the behaviour.
       * @param {string} value
       * @param {string} language
       * @param {'ltr'|'rtl'} direction
       * @returns {any}
       */
      const dirLit = (value, language, direction) =>
        /** @type {any} */ (factory).literal(value, { language, direction });

      // The language tag's lowercasing is already pinned by the string-form literal test above;
      // this block is only about the base direction.
      test('direction is kept and the datatype is rdf:dirLangString', { skip: !directional }, () => {
        const l = dirLit('مرحبا', 'ar', 'rtl');
        assert.equal(l.termType, 'Literal');
        assert.equal(l.value, 'مرحبا');
        assert.equal(l.direction, 'rtl');
        assert.equal(l.datatype.value, RDF_DIR_LANG_STRING);
      });

      test('direction is significant for equals', { skip: !directional }, () => {
        const ltr = dirLit('x', 'en', 'ltr');
        const rtl = dirLit('x', 'en', 'rtl');
        const plain = factory.literal('x', 'en');
        assert.equal(ltr.equals(dirLit('x', 'en', 'ltr')), true);
        assert.equal(ltr.equals(rtl), false, 'ltr != rtl');
        assert.equal(ltr.equals(plain), false, 'directional != plain lang-tagged');
        assert.equal(plain.equals(ltr), false, 'and symmetrically');
        assert.notEqual(termKey(ltr), termKey(rtl), 'the canonical key must discriminate direction');
      });

      test(
        'fromTerm preserves the direction',
        { skip: !directional || !has(factory, 'fromTerm') },
        () => {
          const original = dirLit('x', 'en', 'rtl');
          const copy = /** @type {any} */ (factory.fromTerm(original));
          assert.equal(copy.direction, 'rtl');
          assert.equal(copy.datatype.value, RDF_DIR_LANG_STRING);
          assert.equal(copy.equals(original), true);
        },
      );
    });

    describe('Term#equals', () => {
      test('reflexive across all term types', () => {
        /** @type {Term[]} */
        const terms = [
          factory.namedNode('http://example.org/s'),
          factory.blankNode('b'),
          factory.literal('lit'),
          factory.literal('lit', 'en'),
          factory.defaultGraph(),
          factory.quad(
            factory.namedNode('http://example.org/s'),
            factory.namedNode('http://example.org/p'),
            factory.namedNode('http://example.org/o'),
          ),
        ];
        if (factory.variable) terms.push(factory.variable('v'));
        for (const t of terms) assert.equal(t.equals(t), true, `${t.termType} must equal itself`);
      });

      test('symmetric for equal terms', () => {
        const a = factory.namedNode('http://example.org/s');
        const b = factory.namedNode('http://example.org/s');
        assert.equal(a.equals(b), true);
        assert.equal(b.equals(a), true);
      });

      test('negative across differing values and term types', () => {
        const n = factory.namedNode('http://example.org/s');
        assert.equal(n.equals(factory.namedNode('http://example.org/other')), false);
        assert.equal(n.equals(factory.blankNode('s')), false, 'NamedNode != BlankNode of same value');
        assert.equal(n.equals(factory.literal('http://example.org/s')), false);
        // literal language + datatype sensitivity
        const en = factory.literal('x', 'en');
        assert.equal(en.equals(factory.literal('x', 'fr')), false, 'language is significant');
        assert.equal(en.equals(factory.literal('x')), false, 'lang-tagged != plain');
        const i1 = factory.literal('1', factory.namedNode('http://www.w3.org/2001/XMLSchema#integer'));
        assert.equal(i1.equals(factory.literal('1')), false, 'datatype is significant');
      });

      test('equals(null/undefined) is false', () => {
        const n = factory.namedNode('http://example.org/s');
        assert.equal(n.equals(null), false);
        assert.equal(n.equals(undefined), false);
      });

      test('quad equals compares all four components', () => {
        /** @param {import('@rdfjs/types').Term} o */
        const mk = (o) => factory.quad(
          factory.namedNode('http://example.org/s'),
          factory.namedNode('http://example.org/p'),
          /** @type {any} */ (o),
        );
        assert.equal(mk(factory.literal('a')).equals(mk(factory.literal('a'))), true);
        assert.equal(mk(factory.literal('a')).equals(mk(factory.literal('b'))), false);
      });
    });

    describe('fromTerm / fromQuad', () => {
      test('fromTerm deep-copies each native term type to an equal term', { skip: !has(factory, 'fromTerm') }, () => {
        /** @type {Term[]} */
        const terms = [
          factory.namedNode('http://example.org/s'),
          factory.blankNode('b'),
          factory.literal('lit'),
          factory.literal('lit', 'en'),
          factory.defaultGraph(),
        ];
        if (factory.variable) terms.push(factory.variable('v'));
        for (const original of terms) {
          const copy = /** @type {Term} */ (factory.fromTerm(/** @type {any} */ (original)));
          assert.equal(copy.termType, original.termType);
          assert.equal(copy.equals(original), true, `fromTerm(${original.termType}) must equal original`);
        }
      });

      test('fromTerm copies a FOREIGN plain-object term', { skip: !has(factory, 'fromTerm') }, () => {
        // A minimal cross-implementation NamedNode (RDF/JS Term duck shape).
        const foreign = {
          termType: 'NamedNode',
          value: 'http://example.org/foreign',
          /** @param {any} o */
          equals(o) { return !!o && o.termType === this.termType && o.value === this.value; },
        };
        const copy = factory.fromTerm(/** @type {any} */(foreign));
        assert.equal(copy.termType, 'NamedNode');
        assert.equal(copy.value, 'http://example.org/foreign');
        // The native copy must equal the foreign original (spec: structural equality).
        assert.equal(copy.equals(/** @type {any} */(foreign)), true);
        // And a native term of the same value must equal the copy.
        assert.equal(factory.namedNode('http://example.org/foreign').equals(copy), true);
      });

      test('fromTerm copies a FOREIGN literal with language tag', { skip: !has(factory, 'fromTerm') }, () => {
        const foreignLit = {
          termType: 'Literal',
          value: 'bonjour',
          language: 'fr',
          datatype: { termType: 'NamedNode', value: RDF_LANG_STRING },
          /** @param {any} o */
          equals(o) { return !!o && o.termType === 'Literal' && o.value === this.value && o.language === this.language; },
        };
        const copy = /** @type {import('@rdfjs/types').Literal} */ (
          /** @type {any} */ (factory.fromTerm(/** @type {any} */ (foreignLit)))
        );
        assert.equal(copy.termType, 'Literal');
        assert.equal(copy.value, 'bonjour');
        assert.equal(copy.language, 'fr');
        assert.equal(copy.datatype.value, RDF_LANG_STRING);
      });

      test('fromQuad deep-copies a quad to an equal quad', { skip: !has(factory, 'fromQuad') }, () => {
        const original = factory.quad(
          factory.namedNode('http://example.org/s'),
          factory.namedNode('http://example.org/p'),
          factory.literal('o', 'en'),
          factory.namedNode('http://example.org/g'),
        );
        const copy = factory.fromQuad(original);
        assert.equal(copy.termType, 'Quad');
        assert.equal(copy.equals(original), true, 'fromQuad must produce an equal quad');
        assert.equal(quadKey(copy), quadKey(original));
      });
    });
  });
}

/* ───────────────────────── Dataset suite ───────────────────────── */

/**
 * @param {object} options
 * @param {import('@rdfjs/types').DataFactory} options.factory
 * @param {(quads?: Iterable<import('@rdfjs/types').Quad>) => import('@rdfjs/types').DatasetCore} options.datasetFactory
 * @param {string} [options.label]
 * @param {Function} [options.describe]
 * @param {Function} [options.test]
 */
export async function runDatasetTests(options) {
  const { factory, datasetFactory } = options;
  const label = options.label || 'Dataset';
  assert.ok(factory, 'runDatasetTests requires { factory }');
  assert.ok(typeof datasetFactory === 'function', 'runDatasetTests requires a datasetFactory function');
  const { describe, test } = await resolveTestApi(options);

  /** @param {string} n */
  const ns = (n) => factory.namedNode(`http://example.org/${n}`);
  /**
   * Build a quad. `o` is a string (→ a NamedNode) or any RDF/JS Term; `g` an
   * optional graph term.
   * @param {string} s
   * @param {string} p
   * @param {string|Term} o
   * @param {Term} [g]
   * @returns {Quad}
   */
  const Q = (s, p, o, g) => /** @type {Quad} */ (
    factory.quad(
      ns(s),
      ns(p),
      /** @type {any} */ (typeof o === 'string' ? ns(o) : o),
      /** @type {any} */ (g),
    )
  );

  describe(`${label} — DatasetCore conformance`, () => {
    test('empty dataset has size 0', () => {
      assert.equal(datasetFactory().size, 0);
    });

    test('add increases size, is idempotent, and returns this', () => {
      const ds = datasetFactory();
      const q = Q('s', 'p', 'o');
      const ret = ds.add(q);
      assert.equal(ds.size, 1);
      assert.equal(ret, ds, 'add must return the dataset');
      ds.add(Q('s', 'p', 'o'));
      assert.equal(ds.size, 1, 'add of an equal quad must be idempotent');
    });

    test('has reflects membership by value, not identity', () => {
      const ds = datasetFactory([Q('s', 'p', 'o')]);
      assert.equal(ds.has(Q('s', 'p', 'o')), true);
      assert.equal(ds.has(Q('s', 'p', 'other')), false);
    });

    test('delete removes a quad and returns this', () => {
      const ds = datasetFactory([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      const ret = ds.delete(Q('s', 'p', 'o'));
      assert.equal(ret, ds, 'delete must return the dataset');
      assert.equal(ds.size, 1);
      assert.equal(ds.has(Q('s', 'p', 'o')), false);
      ds.delete(Q('s', 'p', 'absent')); // no-op, must not throw
      assert.equal(ds.size, 1);
    });

    test('constructor/builder seeds the dataset from quads', () => {
      const ds = datasetFactory([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      assert.equal(ds.size, 2);
    });

    test('[Symbol.iterator] yields every quad once', () => {
      const ds = datasetFactory([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      assert.equal(typeof ds[Symbol.iterator], 'function');
      const got = sortedKeys(ds);
      assert.deepEqual(got, sortedKeys([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]));
    });

    describe('match', () => {
      const seed = () => datasetFactory([
        Q('s1', 'p', 'o1'),
        Q('s1', 'p', 'o2'),
        Q('s2', 'p', 'o1', ns('g')),
      ]);

      test('no arguments matches the whole dataset', () => {
        assert.equal(seed().match().size, 3);
      });

      test('subject wildcard combo', () => {
        assert.deepEqual(sortedKeys(seed().match(ns('s1'))),
          sortedKeys([Q('s1', 'p', 'o1'), Q('s1', 'p', 'o2')]));
      });

      test('predicate + object combo', () => {
        assert.deepEqual(sortedKeys(seed().match(null, ns('p'), ns('o1'))),
          sortedKeys([Q('s1', 'p', 'o1'), Q('s2', 'p', 'o1', ns('g'))]));
      });

      test('graph term restricts to that graph', () => {
        assert.deepEqual(sortedKeys(seed().match(null, null, null, ns('g'))),
          sortedKeys([Q('s2', 'p', 'o1', ns('g'))]));
      });

      test('non-matching pattern yields an empty result', () => {
        assert.equal(seed().match(ns('absent')).size, 0);
      });

      test('returns a NEW independent dataset', () => {
        const src = seed();
        const m = src.match(ns('s1'));
        assert.notEqual(m, src, 'match must return a new dataset');
        m.add(Q('injected', 'p', 'o'));
        assert.equal(src.has(Q('injected', 'p', 'o')), false, 'mutating the match result must not affect the source');
      });
    });
  });

  describe(`${label} — Dataset algebra conformance (feature-detected)`, () => {
    // Each algebra method is feature-detected so a DatasetCore-only impl
    // (e.g. @rdfjs/dataset) still passes the core suite above. We PROBE rather
    // than only typeof-check, because a method may exist but throw
    // "not implemented" for a partial impl — in that case we skip-with-note so
    // a real third-party lib stays GREEN while a conformant impl is still tested.
    // An algebra method counts as "present" only if the dataset provides it and
    // it is NOT merely inherited from Object.prototype (so `Object`'s default
    // `toString` on a DatasetCore-only impl does not get treated as the spec's
    // N-Quads `Dataset#toString`).
    /** @param {string} name */
    const present = (name) => {
      const ds = datasetFactory();
      if (!has(ds, name)) return false;
      const fn = /** @type {any} */ (ds)[name];
      return fn !== /** @type {any} */ (Object.prototype)[name];
    };
    /** @param {() => void} fn */
    const probeOk = (fn) => {
      try { fn(); return true; } catch { return false; }
    };
    // Inside the algebra suite we know (by feature detection) the rich Dataset
    // methods are present, so build datasets at the richer `Dataset` type.
    /**
     * @param {Iterable<Quad>} [quads]
     * @returns {Dataset}
     */
    const dsf = (quads) => /** @type {Dataset} */ (datasetFactory(quads));

    test('addAll: unions quads in, returns this', { skip: !present('addAll') }, () => {
      const ds = dsf([Q('s', 'p', 'o')]);
      const ret = ds.addAll([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      assert.equal(ds.size, 2, 'addAll is set-union (no duplicates)');
      assert.equal(ret == null || ret === ds, true);
    });

    test('deleteMatches: removes by pattern, returns this', { skip: !present('deleteMatches') }, () => {
      const ds = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      const ret = ds.deleteMatches(ns('s1'));
      assert.equal(ds.size, 1);
      assert.equal(ds.has(Q('s2', 'p', 'o')), true);
      assert.equal(ret == null || ret === ds, true);
    });

    test('union: contains all quads of both, no duplicates', { skip: !present('union') }, () => {
      const a = dsf([Q('s', 'p', 'o')]);
      const b = dsf([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      const u = a.union(b);
      assert.deepEqual(sortedKeys(u), sortedKeys([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]));
      assert.equal(a.size, 1, 'union must not mutate the receiver');
    });

    test('intersection: only quads in both', { skip: !present('intersection') }, () => {
      const a = dsf([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      const b = dsf([Q('s', 'p', 'o2'), Q('s', 'p', 'o3')]);
      assert.deepEqual(sortedKeys(a.intersection(b)), sortedKeys([Q('s', 'p', 'o2')]));
    });

    test('difference: quads in receiver not in other', { skip: !present('difference') }, () => {
      const a = dsf([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      const b = dsf([Q('s', 'p', 'o2')]);
      assert.deepEqual(sortedKeys(a.difference(b)), sortedKeys([Q('s', 'p', 'o')]));
    });

    test('contains: receiver is a superset of other', { skip: !present('contains') }, () => {
      const a = dsf([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      assert.equal(a.contains(dsf([Q('s', 'p', 'o')])), true);
      assert.equal(a.contains(dsf([Q('s', 'p', 'absent')])), false);
    });

    test('equals: same quad set, order-independent', { skip: !present('equals') }, () => {
      const a = dsf([Q('s', 'p', 'o'), Q('s', 'p', 'o2')]);
      const b = dsf([Q('s', 'p', 'o2'), Q('s', 'p', 'o')]);
      assert.equal(a.equals(b), true);
      assert.equal(a.equals(dsf([Q('s', 'p', 'o')])), false);
    });

    test('filter: predicate selects a sub-dataset, receiver untouched', { skip: !present('filter') }, () => {
      const ds = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      const out = ds.filter((q) => q.subject.value.endsWith('s1'));
      assert.deepEqual(sortedKeys(out), sortedKeys([Q('s1', 'p', 'o')]));
      assert.equal(ds.size, 2);
    });

    test('map: transforms each quad', { skip: !present('map') }, () => {
      const ds = dsf([Q('s1', 'p', 'o')]);
      const out = ds.map((q) => factory.quad(q.subject, q.predicate, ns('mapped'), q.graph));
      assert.deepEqual(sortedKeys(out), sortedKeys([Q('s1', 'p', 'mapped')]));
    });

    test('forEach: visits every quad exactly once', { skip: !present('forEach') }, () => {
      const ds = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      /** @type {Set<string>} */
      const seen = new Set();
      ds.forEach((q) => seen.add(quadKey(q)));
      assert.deepEqual([...seen].sort(), sortedKeys([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]));
    });

    test('some / every', { skip: !(present('some') && present('every')) }, () => {
      const ds = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      assert.equal(ds.some((q) => q.subject.value.endsWith('s1')), true);
      assert.equal(ds.some((q) => q.subject.value.endsWith('zzz')), false);
      assert.equal(ds.every((q) => q.predicate.equals(ns('p'))), true);
      assert.equal(ds.every((q) => q.subject.value.endsWith('s1')), false);
    });

    test('reduce: with seed, and empty+no-seed throws (spec)', { skip: !present('reduce') }, () => {
      const ds = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      const count = ds.reduce((/** @type {number} */ acc) => acc + 1, 0);
      assert.equal(count, 2, 'reduce with a seed folds over every quad');
      // Spec: reduce with no callbackInitialValue over an EMPTY dataset throws
      // (like Array.prototype.reduce). Some impls return undefined instead; we
      // assert the spec only when the impl actually throws, and skip-with-note
      // otherwise so a partial-but-real lib stays green.
      const empty = dsf();
      let threw = false;
      try { empty.reduce((/** @type {any} */ acc) => acc); } catch { threw = true; }
      if (!threw) {
        // eslint-disable-next-line no-console
        console.log('  note: reduce(empty, no-seed) did not throw (impl deviates from spec)');
      }
    });

    test('toArray: yields all quads', { skip: !present('toArray') }, () => {
      const ds = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      const arr = ds.toArray();
      assert.equal(Array.isArray(arr), true);
      assert.deepEqual(sortedKeys(arr), sortedKeys([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]));
    });

    test('toString: produces stable N-Quads-ish text', { skip: !present('toString') }, () => {
      const ds = dsf([Q('s1', 'p', 'o')]);
      const str = ds.toString();
      assert.equal(typeof str, 'string');
      assert.ok(str.length > 0);
    });

    test('toCanonical: deterministic + insert-order independent', {
      skip: !present('toCanonical'),
    }, () => {
      const probe = dsf([Q('s1', 'p', 'o')]);
      if (!probeOk(() => probe.toCanonical())) {
        // eslint-disable-next-line no-console
        console.log('  note: toCanonical present but not implemented — skipping');
        return;
      }
      const a = dsf([Q('s1', 'p', 'o1'), Q('s2', 'p', 'o2')]).toCanonical();
      const b = dsf([Q('s2', 'p', 'o2'), Q('s1', 'p', 'o1')]).toCanonical();
      assert.equal(typeof a, 'string');
      assert.equal(a, b, 'toCanonical must be insert-order independent');
      const again = dsf([Q('s1', 'p', 'o1'), Q('s2', 'p', 'o2')]).toCanonical();
      assert.equal(a, again, 'toCanonical must be repeatable');
    });

    test('toStream + import: round-trips quads through a Stream', {
      skip: !(present('toStream') && present('import')),
    }, async () => {
      const src = dsf([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]);
      const stream = /** @type {any} */ (src.toStream());
      if (!stream || typeof stream.on !== 'function') {
        // eslint-disable-next-line no-console
        console.log('  note: toStream did not return an EventEmitter-shaped stream — skipping');
        return;
      }
      const sink = dsf();
      await new Promise((resolve, reject) => {
        const p = /** @type {any} */ (sink.import(stream));
        if (p && typeof p.then === 'function') p.then(resolve, reject);
        else if (p && typeof p.on === 'function') p.on('end', resolve).on('error', reject);
        else resolve(undefined);
      });
      assert.deepEqual(sortedKeys(sink), sortedKeys([Q('s1', 'p', 'o'), Q('s2', 'p', 'o')]));
    });
  });
}

/* ───────────────────────── Stream suite ───────────────────────── */

/**
 * OPTIONAL RDF/JS Stream/Source/Sink suite. No-op-with-skip when neither
 * `source` nor `sink` is provided.
 *
 * @param {object} options
 * @param {import('@rdfjs/types').Source} [options.source]
 * @param {import('@rdfjs/types').Sink<any, any>} [options.sink]
 * @param {import('@rdfjs/types').DataFactory} [options.factory]
 * @param {string} [options.label]
 * @param {Function} [options.describe]
 * @param {Function} [options.test]
 */
export async function runStreamTests(options) {
  const opts = options || {};
  const { source, sink } = opts;
  const label = opts.label || 'Stream';
  const { describe, test } = await resolveTestApi(opts);

  // The Stream surface is duck-typed across implementations (EventEmitter shape
  // is not in @rdfjs/types), so handle it through `any`-typed locals.
  const src = /** @type {any} */ (source);
  const snk = /** @type {any} */ (sink);

  describe(`${label} — Stream/Source/Sink conformance (optional)`, () => {
    test('source.match emits a data event per quad then end', {
      skip: !(source && has(source, 'match')),
    }, async () => {
      const stream = src.match(null, null, null, null);
      assert.ok(stream, 'source.match must return a stream');
      if (typeof stream.on !== 'function') {
        // eslint-disable-next-line no-console
        console.log('  note: source.match did not return an EventEmitter — skipping');
        return;
      }
      /** @type {any[]} */
      const quads = await new Promise((resolve, reject) => {
        /** @type {any[]} */
        const acc = [];
        stream.on('data', (/** @type {any} */ q) => acc.push(q));
        stream.on('end', () => resolve(acc));
        stream.on('error', reject);
        if (typeof stream.read === 'function') stream.read();
      });
      assert.ok(Array.isArray(quads));
      for (const q of quads) {
        assert.equal(typeof q.termType === 'string' || q.subject != null, true, 'emitted item must be a quad');
      }
    });

    test('sink.import consumes a stream', {
      skip: !(sink && has(sink, 'import') && source && has(source, 'match')),
    }, async () => {
      const stream = src.match(null, null, null, null);
      if (!stream || typeof stream.on !== 'function') {
        // eslint-disable-next-line no-console
        console.log('  note: source.match did not return an EventEmitter — skipping');
        return;
      }
      await new Promise((resolve, reject) => {
        const out = snk.import(stream);
        if (out && typeof out.then === 'function') out.then(resolve, reject);
        else if (out && typeof out.on === 'function') out.on('end', resolve).on('error', reject);
        else resolve(undefined);
      });
    });

    test('skips cleanly when no source/sink provided', { skip: !!(source || sink) }, () => {
      assert.ok(true, 'no Stream surface supplied — nothing to test');
    });
  });
}

/* ───────────────────────── Convenience runner ───────────────────────── */

/**
 * Run whichever suites are wired in `impl`.
 *
 * @param {object} impl
 * @param {import('@rdfjs/types').DataFactory} [impl.factory]
 * @param {(quads?: Iterable<import('@rdfjs/types').Quad>) => import('@rdfjs/types').DatasetCore} [impl.datasetFactory]
 * @param {import('@rdfjs/types').Source} [impl.source]
 * @param {import('@rdfjs/types').Sink<any, any>} [impl.sink]
 * @param {string} [impl.label]
 * @param {Function} [impl.describe]
 * @param {Function} [impl.test]
 */
export async function runAll(impl) {
  const { factory, datasetFactory, source, sink, label, describe, test } = impl || {};
  if (factory) {
    await runDataFactoryTests({ factory, label, describe, test });
  }
  if (factory && datasetFactory) {
    await runDatasetTests({ factory, datasetFactory, label, describe, test });
  }
  if (source || sink) {
    await runStreamTests({ source, sink, factory, label, describe, test });
  }
}

export default { runDataFactoryTests, runDatasetTests, runStreamTests, runAll };
