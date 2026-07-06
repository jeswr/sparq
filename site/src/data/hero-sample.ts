// [OPUS-4.8] sq-vw3ax.11 — the HOME hero in-browser runner's sample graph + default query.
//
// This is the data behind the home page's HeroQueryRunner (src/components/home/hero-runner.tsx):
// a tiny, self-contained Turtle graph and a default SELECT that JOINS three relations
// (person → name, person → component, component → line count), AGGREGATES with SUM, and SORTS
// DESC. That shape is the whole honesty point: a join + group-by-sum + order-by result cannot be
// credibly faked by a static site, so when the visitor presses Run and sees these three rows
// computed in-tab, they are watching the real Rust engine (compiled to wasm) actually evaluate.
//
// REAL DATA / HONESTY (load-bearing): the graph is real Turtle parsed by the wasm Store and the
// result rows are produced by the engine at runtime. The `PREVIEW_ROWS` below are the EXPECTED
// answer, shown dimmed with an explicit "Preview — press Run to compute it live" pill in the idle
// state; they are never presented as a computed/live result and carry no timing. No number here
// is a performance/benchmark claim.

/**
 * The sample graph: three contributors (`:ada` / `:grace` / `:lin`) who each `:wrote` one or
 * more engine `:Component`s, and each component's `:linesOfCode` + `:lang "Rust"`. Small on
 * purpose (marketing density) but real — `:ada` wrote TWO components, so her summed total leads
 * the sort, which makes the SUM visibly load-bearing (not just a row-per-person passthrough).
 */
export const HERO_SAMPLE_TURTLE = `@prefix :     <http://sparq.dev/demo/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

:ada   foaf:name "Ada" ;
       :wrote :parser, :planner .
:grace foaf:name "Grace" ;
       :wrote :optimizer .
:lin   foaf:name "Lin" ;
       :wrote :storage .

:parser    a :Component ; :linesOfCode 4200 ; :lang "Rust" .
:planner   a :Component ; :linesOfCode 3100 ; :lang "Rust" .
:optimizer a :Component ; :linesOfCode 5300 ; :lang "Rust" .
:storage   a :Component ; :linesOfCode 6100 ; :lang "Rust" .
`;

/** The wasm engine's RDF format string for {@link HERO_SAMPLE_TURTLE}. */
export const HERO_SAMPLE_FORMAT = "turtle";

/**
 * The prefilled default query: sum each contributor's lines of Rust across every component
 * they wrote, then rank. Joins `foaf:name` + `:wrote` + `:linesOfCode`, groups by name, orders
 * by the summed total descending — three rows, each visibly the product of a join + aggregate.
 */
export const HERO_DEFAULT_QUERY = `PREFIX :     <http://sparq.dev/demo/>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>

SELECT ?name (SUM(?loc) AS ?linesOfRust) WHERE {
  ?dev       foaf:name     ?name .
  ?dev       :wrote        ?component .
  ?component :linesOfCode  ?loc .
}
GROUP BY ?name
ORDER BY DESC(?linesOfRust)`;

/** The variable names the default query projects, in `head.vars` order. */
export const HERO_RESULT_VARS = ["name", "linesOfRust"] as const;

/**
 * The EXPECTED answer, shown dimmed as an honest PREVIEW before the visitor runs the query (Ada
 * 4200+3100=7300, Lin 6100, Grace 5300). Rendered behind a "Preview — press Run to compute it
 * live in your tab" pill; never presented as a computed result. Kept beside the query so a drift
 * between the two is obvious in review — the live engine remains the source of truth at runtime.
 */
export const HERO_PREVIEW_ROWS: { name: string; linesOfRust: number }[] = [
  { name: "Ada", linesOfRust: 7300 },
  { name: "Lin", linesOfRust: 6100 },
  { name: "Grace", linesOfRust: 5300 },
];
