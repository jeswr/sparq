// [OPUS-4.8] sq-xoxu — built-in examples for the live /surface/full-text playground. The
// default is the classic "quick brown fox" walkthrough from skills/full-text-search/SKILL.md
// + the sparq-text-wasm README: a tiny N-Triples corpus of comment literals, queried with
// `text:matches` (AND of tokens) + `text:score` (BM25), ranked. Each example pins the corpus
// (one small document so the positional index stays bounded), the document format, and a
// `text:`-predicate SELECT the page runs as a stateless one-shot.

export interface TextExample {
  id: string;
  label: string;
  description: string;
  /** The RDF corpus to index (kept tiny so the positions-enabled index stays bounded). */
  data: string;
  /** Document syntax: "turtle" | "ntriples" | "nquads" | "trig". */
  format: string;
  /** The `text:`-predicate SELECT to run over the corpus. */
  query: string;
}

// The quick-brown-fox corpus the SKILL.md / README walkthrough uses, extended to a handful
// of short literals so BM25 ranking and matchesAny / phrase / near have something to rank.
const FOX_CORPUS = `<http://ex/a> <http://ex/comment> "The quick brown fox jumps over the lazy dog" .
<http://ex/b> <http://ex/comment> "A quick red fox" .
<http://ex/c> <http://ex/comment> "The lazy brown dog sleeps" .
<http://ex/d> <http://ex/comment> "Foxes are quick and clever animals" .
<http://ex/e> <http://ex/comment> "Slow and steady wins the race" .`;

const TEXT_PREFIX = "PREFIX text: <http://sparq.dev/text#>";

export const TEXT_EXAMPLES: TextExample[] = [
  {
    id: "matches-score",
    label: "BM25-ranked matches",
    description:
      "text:matches (AND of tokens) + text:score — keyword → BM25-ranked literals. Only literals containing BOTH 'quick' and 'fox' match; ranked by relevance.",
    data: FOX_CORPUS,
    format: "ntriples",
    query: `${TEXT_PREFIX}
SELECT ?s ?lit ?score WHERE {
  ?s <http://ex/comment> ?lit .
  ?lit text:matches "quick fox" ;
       text:score   ?score .
} ORDER BY DESC(?score)`,
  },
  {
    id: "matches-any",
    label: "matchesAny (OR)",
    description:
      "text:matchesAny — literals containing AT LEAST ONE token, BM25-ranked. 'fox dog' matches every literal mentioning either word.",
    data: FOX_CORPUS,
    format: "ntriples",
    query: `${TEXT_PREFIX}
SELECT ?s ?lit ?score WHERE {
  ?s <http://ex/comment> ?lit .
  ?lit text:matchesAny "fox dog" ;
       text:score      ?score .
} ORDER BY DESC(?score)`,
  },
  {
    id: "prefix",
    label: "Prefix token (fox*)",
    description:
      "A token ending in '*' is a prefix query: 'fox*' matches 'fox' and 'foxes'. Combined here with 'quick' as an AND.",
    data: FOX_CORPUS,
    format: "ntriples",
    query: `${TEXT_PREFIX}
SELECT ?s ?lit ?score WHERE {
  ?s <http://ex/comment> ?lit .
  ?lit text:matches "quick fox*" ;
       text:score   ?score .
} ORDER BY DESC(?score)`,
  },
  {
    id: "phrase",
    label: "Phrase (adjacent, in order)",
    description:
      "text:phrase matches only where the tokens are adjacent and in order — needs the positions-enabled index this bundle always builds. 'quick brown' matches; 'brown quick' would not.",
    data: FOX_CORPUS,
    format: "ntriples",
    query: `${TEXT_PREFIX}
SELECT ?s ?lit WHERE {
  ?s <http://ex/comment> ?lit .
  ?lit text:phrase "quick brown" .
}`,
  },
  {
    id: "near",
    label: "Proximity (near + slop)",
    description:
      "text:near is the scored, bounded-gap variant of text:phrase: tokens in order within a gap budget (text:slop), ranked tightest-first (score = 1/(1+gap)). 'quick fox' within slop 2.",
    data: FOX_CORPUS,
    format: "ntriples",
    query: `${TEXT_PREFIX}
SELECT ?s ?lit ?score WHERE {
  ?s <http://ex/comment> ?lit .
  ?lit text:near  "quick fox" ;
       text:slop  2 ;
       text:score ?score .
} ORDER BY DESC(?score)`,
  },
];
