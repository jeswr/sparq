import type { Metadata } from "next";
import { Search } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { TextPlayground } from "@/components/text-playground";

export const metadata: Metadata = {
  title: "Full-text",
  description:
    "BM25 full-text search over RDF string literals with the sparq engine — text: magic predicates (matches / matchesAny / phrase / near / slop / score) inside plain SPARQL, the quick-brown-fox example, live in your tab via the W-text wasm bundle.",
};

export default function FullTextSurfacePage() {
  return (
    <SurfaceContent
      icon={Search}
      title="Full-text"
      statement="Keyword, prefix, phrase and proximity search over RDF literals — BM25-ranked, expressed as text: magic predicates inside ordinary SPARQL."
      tier="live-new-wasm"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-text</code> builds an{" "}
            owned <strong className="text-foreground">BM25 inverted index</strong> over
            the string literals of a{" "}
            <code className="font-mono">sparq_core::Graph</code> and exposes it through{" "}
            <strong className="text-foreground">
              <code className="font-mono">text:</code> magic predicates
            </strong>{" "}
            (<code className="font-mono">http://sparq.dev/text#</code>) that you write
            inside ordinary SPARQL. A keyword search becomes{" "}
            <code className="font-mono">?lit text:matches &quot;quick fox&quot;</code>;
            the relevance score binds with{" "}
            <code className="font-mono">text:score ?s</code>. The rewrite splices the
            index hits into the query as inline{" "}
            <code className="font-mono">VALUES</code> — the SPARQL engine itself is
            unaware of text search.
          </p>
          <p>
            Because it is pure Rust over{" "}
            <code className="font-mono">sparq-engine</code>, it compiles to wasm. The
            playground below loads the tier-b{" "}
            <strong className="text-foreground">&ldquo;W-text&rdquo; bundle</strong> on
            demand — a separate, lazily-loaded bundle from the lean SPARQL REPL — and
            runs the indexer + query in this tab: edit the tiny corpus, run a{" "}
            <code className="font-mono">text:</code> query, see the BM25-ranked literals.
            The default is the classic quick-brown-fox example.
          </p>
        </>
      }
      capabilities={[
        {
          title: "text:matches (AND) + BM25 score",
          body: "Keyword search: literals containing every token, ranked by BM25 relevance bound with text:score.",
        },
        {
          title: "text:matchesAny (OR)",
          body: "Literals containing at least one token — the inclusive variant, also BM25-ranked.",
        },
        {
          title: "Prefix tokens (foo*)",
          body: "A token ending in '*' is a prefix query: 'fox*' matches 'fox' and 'foxes' — autocomplete-style.",
        },
        {
          title: "text:phrase (adjacent, in order)",
          body: "Exact phrase match over the positions-enabled index this bundle always builds; order is significant.",
        },
        {
          title: "text:near + text:slop (proximity)",
          body: "Bounded-gap, in-order proximity match, ranked tightest-first (score = 1/(1+gap)); slop sets the gap budget.",
        },
        {
          title: "Index footprint reported",
          body: "Indexed-literal and distinct-token counts plus an in-memory heap estimate — the index-build-memory surface, shown per run.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Live in your browser tab.</strong> The
            BM25 index + the <code className="font-mono">text:</code> rewrite are pure
            Rust over the engine, compiled to wasm in the separate W-text bundle (
            <code className="font-mono">sparq-text-wasm</code>). The page lazy-loads it on
            first interaction so the landing page never pays for it, then calls the
            bundle&rsquo;s stateless{" "}
            <code className="font-mono text-foreground">TextSearch.query</code> and{" "}
            <code className="font-mono">TextSearch.indexStats</code> bindings — each parses
            the corpus, builds a positions-enabled index, and returns canonical SPARQL 1.1
            JSON (or the footprint summary). Nothing is sent to a server.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            Each search is a stateless one-shot: the bundle rebuilds the index from the
            corpus it is given every call, which keeps the JS side from holding a
            long-lived handle but means it is sized for the illustrative corpora here
            (tens of literals), not a large persistent index. The tokenizer is UAX&nbsp;#29
            word segmentation + Unicode lowercasing — no stemming, no stopword list, no
            diacritic folding (<code className="font-mono">café</code> ≠{" "}
            <code className="font-mono">cafe</code>). Build a real index over a large graph
            natively via <code className="font-mono">sparq-cli</code>, and persist /
            incrementally update it with the native crate&rsquo;s{" "}
            <code className="font-mono">apply_delta</code> / reconcile paths.
          </p>
        </>
      }
      links={[
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-text",
          label: "sparq-text crate",
          external: true,
        },
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-text-wasm",
          label: "W-text wasm bundle",
          external: true,
        },
      ]}
    >
      <TextPlayground />
    </SurfaceContent>
  );
}
