// [OPUS-4.8] sq-lii76 (#981) — the `<script type="module">` ESM-import recipe for the named
// `Dataset` entry. A static (server-rendered) doc card: it shows that the same RDF/JS surface
// the live demo above exercises can be imported by NAME into a bare HTML page from an ESM CDN,
// with the ~MB wasm fetched LAZILY by the first `await Dataset.…` (not by the import line).
// This is the literal snippet shape the maintainer's issue asked for.

import { FileCode2 } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

const SNIPPET = `<script type="module">
  // From any ESM CDN (or a bundler import). The ~MB engine wasm is fetched
  // LAZILY by the first \`await Dataset.fromString(...)\` below — NOT by this
  // import — so it never blocks the page paint.
  import { Dataset, DataFactory as DF } from "https://esm.sh/@jeswr/sparq";

  const ds = await Dataset.fromString(
    '<http://ex/a> <http://ex/name> "Alice" .',
    "ntriples",
  );
  console.log(ds.size); // 1  (DatasetCore: size, add, delete, has, match, iterate)

  ds.add(DF.quad(
    DF.namedNode("http://ex/b"), DF.namedNode("http://ex/name"), DF.literal("Bob"),
  ));
  for (const q of ds.match(null, DF.namedNode("http://ex/name"), null)) {
    console.log(q.subject.value, "→", q.object.value);
  }

  // Drop to the full SPARQL engine when DatasetCore is not enough:
  console.log(ds.store.queryBoolean("ASK { ?s ?p ?o }")); // true
  ds.free();
</script>`;

const GLUE_SNIPPET = `<script type="module">
  // Low-level: the wasm-pack \`--target web\` glue is itself a real ESM module.
  import init, { Store } from "https://jeswr.github.io/sparq/wasm/sparq_wasm.js";
  await init(); // lazily fetches + instantiates sparq_wasm_bg.wasm
  const store = Store.load("<a> <b> <c> .", "ntriples");
</script>`;

export function EsmImportSnippet() {
  return (
    <Card>
      <CardHeader className="flex-row items-start gap-2 space-y-0">
        <FileCode2 className="mt-0.5 size-5 text-primary" aria-hidden="true" />
        <div>
          <CardTitle className="text-base">
            Import <code className="font-mono">Dataset</code> from a{" "}
            <code className="font-mono">&lt;script type=&quot;module&quot;&gt;</code>
          </CardTitle>
          <CardDescription>
            The named <code className="font-mono">Dataset</code> export is an RDF/JS{" "}
            <code className="font-mono">DatasetCore</code> over the engine — importable
            directly in a browser module script from any ESM CDN. The async factories
            (<code className="font-mono">Dataset.fromString</code> /{" "}
            <code className="font-mono">.create</code> /{" "}
            <code className="font-mono">.fromQuads</code>) instantiate the wasm engine on
            first use, so the ~MB binary loads lazily — never on import.
          </CardDescription>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
          {SNIPPET}
        </pre>
        <p className="text-sm text-muted-foreground">
          Prefer <code className="font-mono">Dataset</code> (or{" "}
          <code className="font-mono">SparqStore</code>) in an app — the cold start is
          memoised, so it is paid at most once per page. The lower-level engine handle is
          also importable: the wasm-pack glue is itself an ESM module.
        </p>
        <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
          {GLUE_SNIPPET}
        </pre>
      </CardContent>
    </Card>
  );
}
