/**
 * Minimal, spec-compliant RDF/JS data model (hand-rolled to keep the package
 * dependency-free at runtime; typed against `@rdfjs/types`).
 */
import type * as RDF from '@rdfjs/types';

export class NamedNode<Iri extends string = string> implements RDF.NamedNode<Iri> {
  readonly termType = 'NamedNode' as const;
  constructor(readonly value: Iri) {}
  equals(other?: RDF.Term | null): boolean {
    return !!other && other.termType === 'NamedNode' && other.value === this.value;
  }
}

export class BlankNode implements RDF.BlankNode {
  readonly termType = 'BlankNode' as const;
  constructor(readonly value: string) {}
  equals(other?: RDF.Term | null): boolean {
    return !!other && other.termType === 'BlankNode' && other.value === this.value;
  }
}

const XSD_STRING = 'http://www.w3.org/2001/XMLSchema#string';
const RDF_LANG_STRING = 'http://www.w3.org/1999/02/22-rdf-syntax-ns#langString';
const RDF_DIR_LANG_STRING = 'http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString';

export class Literal implements RDF.Literal {
  readonly termType = 'Literal' as const;
  readonly language: string;
  readonly direction: 'ltr' | 'rtl' | '';
  readonly datatype: RDF.NamedNode;

  constructor(
    readonly value: string,
    languageOrDatatype?: string | RDF.NamedNode | { language: string; direction?: 'ltr' | 'rtl' | '' | null } | null,
  ) {
    if (typeof languageOrDatatype === 'string' && languageOrDatatype !== '') {
      this.language = languageOrDatatype.toLowerCase();
      this.direction = '';
      this.datatype = new NamedNode(RDF_LANG_STRING);
    } else if (languageOrDatatype && typeof languageOrDatatype === 'object' && 'termType' in languageOrDatatype) {
      this.language = '';
      this.direction = '';
      this.datatype = languageOrDatatype;
    } else if (languageOrDatatype && typeof languageOrDatatype === 'object') {
      // DirectionalLanguage (RDF 1.2)
      this.language = languageOrDatatype.language.toLowerCase();
      this.direction = languageOrDatatype.direction ?? '';
      this.datatype = new NamedNode(this.direction === '' ? RDF_LANG_STRING : RDF_DIR_LANG_STRING);
    } else {
      this.language = '';
      this.direction = '';
      this.datatype = new NamedNode(XSD_STRING);
    }
  }

  equals(other?: RDF.Term | null): boolean {
    return (
      !!other &&
      other.termType === 'Literal' &&
      other.value === this.value &&
      other.language === this.language &&
      (other.direction ?? '') === this.direction &&
      other.datatype.equals(this.datatype)
    );
  }
}

export class Variable implements RDF.Variable {
  readonly termType = 'Variable' as const;
  constructor(readonly value: string) {}
  equals(other?: RDF.Term | null): boolean {
    return !!other && other.termType === 'Variable' && other.value === this.value;
  }
}

export class DefaultGraph implements RDF.DefaultGraph {
  readonly termType = 'DefaultGraph' as const;
  readonly value = '' as const;
  equals(other?: RDF.Term | null): boolean {
    return !!other && other.termType === 'DefaultGraph';
  }
}

const DEFAULT_GRAPH = new DefaultGraph();

export class Quad implements RDF.Quad {
  readonly termType = 'Quad' as const;
  readonly value = '' as const;
  constructor(
    readonly subject: RDF.Quad_Subject,
    readonly predicate: RDF.Quad_Predicate,
    readonly object: RDF.Quad_Object,
    readonly graph: RDF.Quad_Graph = DEFAULT_GRAPH,
  ) {}
  equals(other?: RDF.Term | null): boolean {
    return (
      !!other &&
      other.termType === 'Quad' &&
      this.subject.equals(other.subject) &&
      this.predicate.equals(other.predicate) &&
      this.object.equals(other.object) &&
      this.graph.equals(other.graph)
    );
  }
}

let blankNodeCounter = 0;

function fromTerm(original: RDF.NamedNode): NamedNode;
function fromTerm(original: RDF.BlankNode): BlankNode;
function fromTerm(original: RDF.Literal): Literal;
function fromTerm(original: RDF.Variable): Variable;
function fromTerm(original: RDF.DefaultGraph): DefaultGraph;
function fromTerm(original: RDF.BaseQuad): Quad;
function fromTerm(original: RDF.Term): RDF.Term;
function fromTerm(original: RDF.Term): RDF.Term {
  switch (original.termType) {
    case 'NamedNode':
      return new NamedNode(original.value);
    case 'BlankNode':
      return new BlankNode(original.value);
    case 'Literal':
      return new Literal(
        original.value,
        original.language !== ''
          ? { language: original.language, direction: original.direction ?? '' }
          : new NamedNode(original.datatype.value),
      );
    case 'Variable':
      return new Variable(original.value);
    case 'DefaultGraph':
      return DEFAULT_GRAPH;
    case 'Quad':
      return fromQuad(original);
  }
}

function fromQuad(original: RDF.BaseQuad): Quad {
  return new Quad(
    fromTerm(original.subject) as RDF.Quad_Subject,
    fromTerm(original.predicate) as RDF.Quad_Predicate,
    fromTerm(original.object) as RDF.Quad_Object,
    fromTerm(original.graph) as RDF.Quad_Graph,
  );
}

/** A spec-compliant RDF/JS DataFactory over the term classes above. */
export const DataFactory: RDF.DataFactory & { variable(value: string): Variable } = {
  namedNode: <Iri extends string = string>(value: Iri) => new NamedNode(value),
  blankNode: (value?: string) => new BlankNode(value ?? `b${++blankNodeCounter}`),
  literal: (value: string, languageOrDatatype?: string | RDF.NamedNode | { language: string; direction?: 'ltr' | 'rtl' | '' | null }) =>
    new Literal(value, languageOrDatatype),
  variable: (value: string) => new Variable(value),
  defaultGraph: () => DEFAULT_GRAPH,
  quad: (subject, predicate, object, graph) => new Quad(subject, predicate, object, graph),
  fromTerm,
  fromQuad,
};
