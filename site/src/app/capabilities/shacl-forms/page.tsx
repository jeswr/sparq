import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "SHACL-driven forms",
  description:
    "See how sparq derives open, DASH-compatible forms from SHACL shapes — live validation, SPARQL UPDATE commits, and native GUI rendering are available today; the MCP agent surface and shape editor are planned.",
};

const shape = `ex:PersonShape a sh:NodeShape ;
  sh:targetClass schema:Person ;
  sh:property [
    sh:path schema:name ;
    sh:name "Name" ;
    sh:minCount 1 ;
    sh:order 1
  ], [
    sh:path schema:birthDate ;
    sh:name "Born" ;
    sh:datatype xsd:date ;
    sh:order 2
  ] .`;

// [claude] sq-lsp7k: updated statuses to reflect shipped capabilities:
// - Live field validation landed in PR #2364 (sq-lsp7k.1.3)
// - Form-diff to SPARQL UPDATE landed in PR #2022 (sparq_forms::to_sparql_update)
// - Native GUI forms IPC bridge landed in PRs #2308/#2300 (sq-1cnox/sq-3eukz)
// Still designed-only: F5 role/hidden/readOnly filtering, F6 MCP describe_form, Phase-2 shape editor.
const architecture = [
  {
    label: "Headless core",
    status: "Implemented and verified",
    detail: "Derives a renderer-neutral form description from data, shapes, and a focus node.",
    active: true,
  },
  {
    label: "Native desktop renderer",
    status: "Available today",
    detail: "The native GUI exposes a forms IPC bridge and derives forms from the active workspace without GUI code in the headless core.",
    active: true,
  },
  {
    label: "Edit + commit",
    status: "Available today",
    detail: "Term-level edits produce a reviewable SPARQL 1.1 UPDATE (via sparq_forms::to_sparql_update); live per-field validation runs before commit.",
    active: true,
  },
  {
    label: "Agent surface",
    status: "Designed, not shipped",
    detail: "MCP tools to expose the shape-aware description and guarded edit path (F6 describe_form) remain planned.",
    active: false,
  },
] as const;

// [GPT-5.6] sq-aocxa — a static, dependency-free capability explainer. The artifact is
// intentionally illustrative: only the headless derivation is labelled as implemented.
export default function ShaclFormsPage() {
  return (
    <main className="mx-auto max-w-6xl space-y-16 py-8 sm:py-14">
      <header className="max-w-3xl space-y-5">
        <p className="text-sm font-semibold uppercase tracking-[0.18em] text-primary">
          SHACL-driven forms
        </p>
        <h1 className="text-balance text-4xl font-semibold tracking-tight sm:text-6xl">
          Model the data once. Derive the form.
        </h1>
        <p className="text-pretty text-lg leading-8 text-muted-foreground">
          sparq turns SHACL shapes into an open form description instead of making people
          edit raw triples. The headless core, live validation, SPARQL UPDATE commit, and
          native desktop renderer are available today; the MCP agent surface and the
          meta-circular shape editor remain planned.
        </p>
        <div className="flex flex-wrap gap-2 text-sm" aria-label="Implementation status">
          <span className="rounded-full border border-primary/40 bg-primary/10 px-3 py-1 font-medium text-primary">
            F1 headless core: implemented
          </span>
          <span className="rounded-full border border-primary/40 bg-primary/10 px-3 py-1 font-medium text-primary">
            Live validation + SPARQL UPDATE: available
          </span>
          <span className="rounded-full border border-primary/40 bg-primary/10 px-3 py-1 font-medium text-primary">
            Native GUI forms bridge: available
          </span>
          <span className="rounded-full border px-3 py-1 text-muted-foreground">
            MCP agent surface + shape editor: planned
          </span>
        </div>
      </header>

      <section aria-labelledby="artifact-title" className="space-y-5">
        <div className="max-w-2xl space-y-2">
          <h2 id="artifact-title" className="text-2xl font-semibold tracking-tight">
            One shape, a form every renderer understands
          </h2>
          <p className="text-muted-foreground">
            The shape stays RDF. The derived fields are renderer-neutral.
          </p>
        </div>

        <div className="grid overflow-hidden rounded-2xl border bg-card shadow-sm lg:grid-cols-2">
          <div className="border-b bg-muted/30 p-5 lg:border-b-0 lg:border-r sm:p-7">
            <p className="mb-4 text-xs font-semibold uppercase tracking-widest text-muted-foreground">
              SHACL shape
            </p>
            <pre className="overflow-x-auto text-sm leading-7" tabIndex={0}>
              <code>{shape}</code>
            </pre>
          </div>

          <div className="p-5 sm:p-7">
            <div className="mb-6 flex items-center justify-between gap-4">
              <div>
                <p className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">
                  Derived form
                </p>
                <h3 className="mt-1 text-xl font-semibold">Person</h3>
              </div>
              <span className="rounded-full bg-muted px-3 py-1 text-xs text-muted-foreground">
                illustrative renderer
              </span>
            </div>
            <div className="space-y-5" aria-label="Illustrative form generated from the shape">
              <label className="block space-y-2">
                <span className="text-sm font-medium">
                  Name <span className="text-primary">required</span>
                </span>
                <span className="block rounded-lg border bg-background px-3 py-2.5 text-muted-foreground">
                  Ada Lovelace
                </span>
              </label>
              <label className="block space-y-2">
                <span className="text-sm font-medium">Born</span>
                <span className="block rounded-lg border bg-background px-3 py-2.5 text-muted-foreground">
                  1815-12-10
                </span>
              </label>
            </div>
          </div>
        </div>
        <p className="text-sm text-muted-foreground">
          Built on the open datashapes.org forms model and DASH vocabulary, so existing
          compatible shape libraries do not need a sparq-specific schema.
        </p>
      </section>

      <section aria-labelledby="why-title" className="grid gap-8 lg:grid-cols-[0.8fr_1.2fr]">
        <div className="space-y-3">
          <h2 id="why-title" className="text-2xl font-semibold tracking-tight">
            The missing layer in RDF engines
          </h2>
          <p className="leading-7 text-muted-foreground">
            Engine vendors generally stop at validation, leaving authoring to raw triples or
            SPARQL. TopBraid EDG proves shapes-to-forms is valuable, but it is a product suite,
            not an embeddable engine. sparq takes the open-spec route.
          </p>
        </div>

        <ol className="grid gap-3 sm:grid-cols-2">
          {architecture.map((item) => (
            <li key={item.label} className="rounded-xl border bg-card p-5">
              <p className="font-semibold">{item.label}</p>
              <p className={item.active ? "mt-1 text-sm text-primary" : "mt-1 text-sm text-muted-foreground"}>
                {item.status}
              </p>
              <p className="mt-3 text-sm leading-6 text-muted-foreground">{item.detail}</p>
            </li>
          ))}
        </ol>
      </section>

      <details className="rounded-xl border bg-card p-5 open:shadow-sm sm:p-6">
        <summary className="cursor-pointer font-semibold focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
          How the pieces fit together
        </summary>
        <div className="mt-5 max-w-3xl space-y-4 text-sm leading-7 text-muted-foreground">
          <p>
            The opt-in, GUI-free core accepts a data graph, shapes graph, focus node, mode,
            and role. It emits groups and fields with labels, DASH widgets, values,
            constraints, and editability. Web and desktop hosts can render that same result.
          </p>
          <p>
            Live per-field validation, guarded SPARQL UPDATE commits (via{" "}
            <code>sparq_forms::to_sparql_update</code>), and the native GUI forms bridge
            are available today. What remains planned: role-based and hidden/readOnly field
            filtering (F5), the MCP <code>describe_form</code> agent tool (F6), and the
            Phase-2 meta-circular shape editor.
          </p>
          <a
            href="https://datashapes.org/forms.html"
            className="inline-flex font-medium text-primary underline-offset-4 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            Read the open forms specification
          </a>
        </div>
      </details>
    </main>
  );
}
