/**
 * An immutable RDF/JS `Bindings` implementation (the Map-like solution row of
 * the RDF/JS Query specification), backed by a `Map` keyed on variable name.
 */
import type * as RDF from '@rdfjs/types';
import { Variable } from './terms.js';

type Entry = [RDF.Variable, RDF.Term];

function keyName(key: RDF.Variable | string): string {
  return typeof key === 'string' ? key : key.value;
}

export class Bindings implements RDF.Bindings {
  readonly type = 'bindings' as const;
  private readonly entriesMap: Map<string, Entry>;

  constructor(entries: Iterable<Entry> = []) {
    this.entriesMap = new Map();
    for (const [variable, term] of entries) {
      this.entriesMap.set(variable.value, [variable, term]);
    }
  }

  has(key: RDF.Variable | string): boolean {
    return this.entriesMap.has(keyName(key));
  }

  get(key: RDF.Variable | string): RDF.Term | undefined {
    return this.entriesMap.get(keyName(key))?.[1];
  }

  set(key: RDF.Variable | string, value: RDF.Term): Bindings {
    const variable = typeof key === 'string' ? new Variable(key) : key;
    const next = new Bindings(this.entriesMap.values());
    next.entriesMap.set(variable.value, [variable, value]);
    return next;
  }

  delete(key: RDF.Variable | string): Bindings {
    const next = new Bindings(this.entriesMap.values());
    next.entriesMap.delete(keyName(key));
    return next;
  }

  *keys(): IterableIterator<RDF.Variable> {
    for (const [variable] of this.entriesMap.values()) yield variable;
  }

  *values(): IterableIterator<RDF.Term> {
    for (const [, term] of this.entriesMap.values()) yield term;
  }

  entries(): IterableIterator<Entry> {
    return this.entriesMap.values();
  }

  forEach(fn: (value: RDF.Term, key: RDF.Variable) => unknown): void {
    for (const [variable, term] of this.entriesMap.values()) fn(term, variable);
  }

  get size(): number {
    return this.entriesMap.size;
  }

  [Symbol.iterator](): Iterator<Entry> {
    return this.entriesMap.values();
  }

  equals(other: RDF.Bindings | null | undefined): boolean {
    if (!other) return false;
    if (other.size !== this.size) return false;
    for (const [variable, term] of this.entriesMap.values()) {
      const otherTerm = other.get(variable);
      if (!otherTerm || !otherTerm.equals(term)) return false;
    }
    return true;
  }

  filter(fn: (value: RDF.Term, key: RDF.Variable) => boolean): Bindings {
    const next: Entry[] = [];
    for (const [variable, term] of this.entriesMap.values()) {
      if (fn(term, variable)) next.push([variable, term]);
    }
    return new Bindings(next);
  }

  map(fn: (value: RDF.Term, key: RDF.Variable) => RDF.Term): Bindings {
    const next: Entry[] = [];
    for (const [variable, term] of this.entriesMap.values()) {
      next.push([variable, fn(term, variable)]);
    }
    return new Bindings(next);
  }

  /**
   * [OPUS-4.8] #1123 — an OXIGRAPH-shaped view of this solution: a plain `Map<string, Term>`
   * keyed on the variable NAME (no `?`), the exact shape Oxigraph's JS `Store.query` yields per
   * solution. This is the drop-in for Oxigraph-migration code that does `binding.get("s")` /
   * `for (const [name, term] of binding)` — a `Bindings` already answers `.get("s")` directly,
   * but its `[Symbol.iterator]` / `.keys()` yield RDF/JS `Variable`s (not bare strings) per the
   * RDF/JS Query spec, so `toMap()` is the bridge when the iteration shape must match Oxigraph's
   * `Map<string, Term>`. The conversion is O(size) and allocates one `Map` — no wasm round-trip.
   */
  toMap(): Map<string, RDF.Term> {
    const map = new Map<string, RDF.Term>();
    for (const [variable, term] of this.entriesMap.values()) map.set(variable.value, term);
    return map;
  }

  merge(other: RDF.Bindings): Bindings | undefined {
    const next = new Bindings(this.entriesMap.values());
    for (const [variable, term] of other) {
      const existing = next.entriesMap.get(variable.value);
      if (existing && !existing[1].equals(term)) return undefined;
      next.entriesMap.set(variable.value, [variable, term]);
    }
    return next;
  }

  mergeWith(
    merger: (self: RDF.Term, other: RDF.Term, key: RDF.Variable) => RDF.Term,
    other: RDF.Bindings,
  ): Bindings {
    const next = new Bindings(this.entriesMap.values());
    for (const [variable, term] of other) {
      const existing = next.entriesMap.get(variable.value);
      const merged = existing && !existing[1].equals(term) ? merger(existing[1], term, variable) : term;
      next.entriesMap.set(variable.value, [variable, merged]);
    }
    return next;
  }
}
