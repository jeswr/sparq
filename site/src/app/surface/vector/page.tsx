import type { Metadata } from "next";
import { Binary } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { VectorWalkthrough } from "@/components/vector-walkthrough";

export const metadata: Metadata = {
  title: "Vector / ANN",
  description:
    "sparq-vectors — an opt-in memory-mapped per-term-id embedding store (.spqv) with exact cosine top-k, on-disk DiskANN/Vamana and in-RAM HNSW nearest-neighbour search, predicate-constrained (filtered) ANN, hybrid RRF fusion, and k-NN inside plain SPARQL via the vec: magic predicates. This page replays REAL captured engine output: the Usain Bolt label-embedding run and the vec:nearest / vec:search results, byte-for-byte.",
};

export default function VectorSurfacePage() {
  return (
    <SurfaceContent
      icon={Binary}
      title="Vector / ANN"
      statement="A memory-mapped per-term-id embedding store with exact + approximate (HNSW / DiskANN) nearest-neighbour search, predicate-constrained ANN, hybrid fusion, and k-NN inside plain SPARQL."
      tier="walkthrough"
      extraBadge="Opt-in crate · captured output"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-vectors</code> adds embedding
            storage + nearest-neighbour (ANN) search to the engine. It stores{" "}
            <strong className="text-foreground">one f32 vector per dictionary term id</strong>{" "}
            in a flat memory-mapped <code className="font-mono">.spqv</code> file (sparse by
            design — entities, not every literal), then queries top-
            <code className="font-mono">k</code> by{" "}
            <strong className="text-foreground">cosine</strong>: an exact brute-force scan,
            a persistent on-disk <strong className="text-foreground">DiskANN / Vamana</strong>{" "}
            graph (<code className="font-mono">.spqg</code>), or an in-RAM{" "}
            <strong className="text-foreground">HNSW</strong> index behind the opt-in{" "}
            <code className="font-mono">approx-ann</code> feature. Embeddings are produced{" "}
            <em>out of process</em> — the crate verbalizes entities to text and embeds via a
            provider-agnostic trait; it never runs a model itself.
          </p>
          <p>
            The RDF-native differentiators: <strong className="text-foreground">k-NN
            inside plain SPARQL</strong> via the <code className="font-mono">vec:nearest</code>{" "}
            / <code className="font-mono">vec:search</code> magic predicates (a
            spargebra-algebra rewrite — the engine and wasm bundle are unchanged),{" "}
            <strong className="text-foreground">predicate-constrained (filtered) ANN</strong>{" "}
            that searches only the graph nodes a BGP admits, and{" "}
            <strong className="text-foreground">hybrid fusion</strong> (RRF / score blend)
            with another ranked signal.
          </p>
          <p>
            It is an <strong className="text-foreground">opt-in crate</strong>: nothing in
            the workspace (or the lean wasm bundle) depends on it, the default build does not
            compile it, and the <code className="font-mono">vec:</code> predicate sits behind
            the non-default <code className="font-mono">vec-predicate</code> feature. This
            static Pages site has no backend and the crate is native-only, so &mdash;
            honestly &mdash; this is a{" "}
            <strong className="text-foreground">captured-output walkthrough</strong>:{" "}
            <strong className="text-foreground">(1)</strong> the Usain Bolt
            label-embedding run and <strong className="text-foreground">(2)</strong> every{" "}
            <code className="font-mono">vec:</code> result table are{" "}
            <em>real, verbatim engine output</em> &mdash; produced by running the real
            binary over a tiny declared fixture with the answer-exact backend (term
            serialization, datatypes intact). The cosines are the engine&rsquo;s exact f32.
            The embeddings come from the deterministic{" "}
            <strong className="text-foreground">test-only</strong>{" "}
            <code className="font-mono">HashEmbedder</code> (lexical hashing, no semantics),
            so they demonstrate the <em>pipeline and the exact geometry</em>, not the
            retrieval quality of a real model.
          </p>
        </>
      }
      capabilities={[
        {
          term: "Memory-mapped per-term-id store (.spqv)",
          body: "One f32 vector per dictionary id; get is one binary search + a contiguous slice. Sparse by design, corrupt files rejected up front, build bigger than RAM with StreamingWriter.",
        },
        {
          term: "Exact + approximate search",
          body: "nearest_exact is the answer-exact ground truth; the on-disk DiskANN/Vamana graph (build once, reopen with no rebuild) and the opt-in in-RAM HNSW trade recall (< 1.0) for speed at scale.",
        },
        {
          term: "k-NN inside plain SPARQL (opt-in vec-predicate)",
          body: "vec:nearest / vec:search run vector k-NN via a spargebra-algebra rewrite that inlines the hits as a VALUES table — the engine, planner and wasm bundle are unchanged.",
        },
        {
          term: "Predicate-constrained (filtered) ANN (opt-in filtered-ann)",
          body: "Restrict the search to the graph nodes a SPARQL BGP admits — the join-connected sub-BGP of the neighbour variable derives the candidate mask; a cost model picks pre- vs post-filter, both byte-identical.",
        },
        {
          term: "Hybrid fusion + quantization",
          body: "fuse_rrf / fuse_scores combine text vectors with another ranked signal (e.g. structural similarity); ScalarQuantizer (4×) and ProductQuantizer (8–32×) shrink large stores.",
        },
        {
          term: "Bring-your-own embeddings",
          body: "Embeddings are out-of-process: import a matrix computed elsewhere (NumPy .npy / flat f32 dump) keyed by dict id, or wire a real OpenAI-compatible endpoint behind the opt-in embeddings feature. The default build opens no sockets.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Captured-output replay.</strong> The Bolt
            neighbour list and every <code className="font-mono">vec:</code> result table are
            the verbatim output of the real <code className="font-mono">sparq-vectors</code>{" "}
            binary (the capture harness{" "}
            <code className="font-mono">examples/capture_surface_vector.rs</code>, built with{" "}
            <code className="font-mono">--features vec-predicate</code>) over a tiny declared
            in-memory fixture, using the <strong className="text-foreground">answer-exact</strong>{" "}
            scan — no index build, no model, no downloaded data. Everything is deterministic
            (fixed lexical embedder, exact backend, id-ascending ties), so re-running the
            harness yields byte-identical output; a pinning test (
            <code className="font-mono">site/test/vector.test.mjs</code>) asserts the
            serialization can&rsquo;t drift. To run it for yourself:
          </p>
          <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
            {`cargo run -p sparq-vectors --features vec-predicate \\
  --example capture_surface_vector`}
          </pre>
        </>
      }
      caveat={
        <>
          <p>
            <strong className="text-foreground">HashEmbedder is test-only.</strong> The
            captured embeddings come from a deterministic lexical n-gram hash with{" "}
            <em>no semantics</em> (&ldquo;car&rdquo; and &ldquo;automobile&rdquo; are
            unrelated) — the neighbours are real and reproducible, but they show the
            store / search / SPARQL-rewrite <em>pipeline</em> and the exact geometry, not
            the retrieval quality of a real embedding model (a deployment supplies its own{" "}
            <code className="font-mono">Embedder</code>).
          </p>
          <p>
            <strong className="text-foreground">
              Approximate search is approximate; recall &lt; 1.0.
            </strong>{" "}
            Only the exact scan and these captured runs are answer-exact. The HNSW / DiskANN
            recall figures referenced here are the crate&rsquo;s own{" "}
            <code className="font-mono">cargo test</code> gates &mdash;{" "}
            <strong className="text-foreground">representative, not canonical</strong> and not
            measured on this page; run the tests for the numbers. No latency figure is shown
            (hardware-dependent). The <code className="font-mono">vec:</code> predicate,
            filtered ANN, HNSW and live embeddings are all <em>opt-in</em> cargo features,
            off in the default build.
          </p>
        </>
      }
      links={[
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-vectors",
          label: "sparq-vectors crate",
          external: true,
        },
        {
          href: "https://github.com/jeswr/sparq/blob/main/skills/vector-search/SKILL.md",
          label: "vector-search SKILL.md",
          external: true,
        },
      ]}
    >
      <VectorWalkthrough />
    </SurfaceContent>
  );
}
