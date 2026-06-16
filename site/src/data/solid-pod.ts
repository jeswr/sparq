// [OPUS-4.8] sq-4r4b — the fixture Solid Pod + WAC/ACP grants + (user, app) sessions
// for the Solid (user, app)-pair flagship (`/showcase/solid-pairs`).
//
// HONESTY (load-bearing — mirror this in the page UI): the WAC/ACP *materialization*
// engine (the real `sparq-solid` crate + its N3 reasoner) runs NATIVELY / at build
// time, not in the browser. What runs LIVE in your tab is the resulting SPARQL
// `FROM NAMED <authorized-graphs>` dataset restriction — which is the real engine
// doing the real restriction. This faithfully mirrors `sparq-solid`'s
// `query_as_rewrite` path: the per-(agent, client) authorized named-graph set is
// the materialized decision; the same query rewritten with that set is what the
// browser evaluates. See research/solid-access-control-design.md + sparq-solid-scope.md.
//
// The Pod is modelled exactly as `sparq-solid` models a Pod: ONE NAMED GRAPH PER
// DOCUMENT, the graph name == the resource IRI. Access control is at the named-graph
// level; a graph with no applicable grant is INVISIBLE (fail-closed: absence = deny,
// design decision D4). A non-authorized graph is indistinguishable from an absent one.

/** A document in the Pod = one named graph. `iri` is the graph name. */
export interface PodDoc {
  iri: string;
  /** Short label for the UI. */
  label: string;
  /** One-line description of what it holds. */
  about: string;
  /** Sensitivity class, drives the UI colour. */
  sensitivity: "public" | "shared" | "private";
  /** The N-Quads body (each line a quad in graph <iri>). */
  quads: string;
}

/** The WebID prefix for the Pod owner. */
const POD = "https://alice.pod.example/";
const OWNER = "https://alice.pod.example/profile/card#me";
const FRIEND = "https://bob.example/profile#me";

// ----------------------------------------------------------------------------
// The Pod documents — one named graph each. Plain, queryable RDF.
// ----------------------------------------------------------------------------

/** Public profile — name, role, public links. Readable by everyone. */
const DOC_PROFILE: PodDoc = {
  iri: `${POD}profile/card`,
  label: "Public profile",
  about: "Name, role and public links — Alice's WebID profile document.",
  sensitivity: "public",
  quads: [
    `<${OWNER}> <http://xmlns.com/foaf/0.1/name> "Alice Okafor" <${POD}profile/card> .`,
    `<${OWNER}> <http://xmlns.com/foaf/0.1/title> "Hydrologist" <${POD}profile/card> .`,
    `<${OWNER}> <http://xmlns.com/foaf/0.1/homepage> <https://alice.example/> <${POD}profile/card> .`,
  ].join("\n"),
};

/** Shared calendar — events Alice shares with friends. Owner + friends (any app). */
const DOC_CALENDAR: PodDoc = {
  iri: `${POD}calendar/shared`,
  label: "Shared calendar",
  about: "Events Alice shares with friends — book club, a hiking trip.",
  sensitivity: "shared",
  quads: [
    `<${POD}calendar/shared#e1> <http://schema.org/name> "Book club" <${POD}calendar/shared> .`,
    `<${POD}calendar/shared#e1> <http://schema.org/startDate> "2026-07-02"^^<http://www.w3.org/2001/XMLSchema#date> <${POD}calendar/shared> .`,
    `<${POD}calendar/shared#e2> <http://schema.org/name> "Brecon hike" <${POD}calendar/shared> .`,
    `<${POD}calendar/shared#e2> <http://schema.org/startDate> "2026-07-19"^^<http://www.w3.org/2001/XMLSchema#date> <${POD}calendar/shared> .`,
  ].join("\n"),
};

/** Private health data — only the owner, and only from her trusted health app. */
const DOC_HEALTH: PodDoc = {
  iri: `${POD}health/records`,
  label: "Private health data",
  about: "Sensitive medical records — restricted to the owner via one trusted app.",
  sensitivity: "private",
  quads: [
    `<${POD}health/records#bp> <http://schema.org/name> "Blood pressure" <${POD}health/records> .`,
    `<${POD}health/records#bp> <http://schema.org/value> "118/76" <${POD}health/records> .`,
    `<${POD}health/records#hr> <http://schema.org/name> "Resting heart rate" <${POD}health/records> .`,
    `<${POD}health/records#hr> <http://schema.org/value> "61 bpm" <${POD}health/records> .`,
  ].join("\n"),
};

/** Private notes — only the owner, any app. Never shared. */
const DOC_NOTES: PodDoc = {
  iri: `${POD}notes/private`,
  label: "Private notes",
  about: "Personal notes — owner only, never shared with anyone.",
  sensitivity: "private",
  quads: [
    `<${POD}notes/private#n1> <http://schema.org/text> "Renew passport before August" <${POD}notes/private> .`,
    `<${POD}notes/private#n2> <http://schema.org/text> "Idea: river-flow dataset for the talk" <${POD}notes/private> .`,
  ].join("\n"),
};

export const POD_DOCS: PodDoc[] = [
  DOC_PROFILE,
  DOC_CALENDAR,
  DOC_HEALTH,
  DOC_NOTES,
];

/** The whole Pod as one N-Quads document (named graphs preserved). */
export const POD_NQUADS: string = POD_DOCS.map((d) => d.quads).join("\n") + "\n";

// ----------------------------------------------------------------------------
// The access-control rules (WAC / ACP). These are what `sparq-solid` materializes
// natively at build time into the per-(agent, client) authorized graph set. We
// surface them in the UI as the WHY behind each result — the grant that produced it.
// ----------------------------------------------------------------------------

export type AclSystem = "WAC" | "ACP";

/** One human-readable access-control grant attached to a document. */
export interface AclGrant {
  /** Which document (named graph) the grant governs. */
  docIri: string;
  system: AclSystem;
  /** Who the grant is for, in display form. */
  subject: string;
  /** Which app/client origin the grant is scoped to ("any app" = no client restriction). */
  client: string;
  /** The acl: / acp: modes granted. */
  modes: string[];
  /** The real rule snippet (WAC Turtle / ACP), for the "show me the rule" panel. */
  rule: string;
}

// WAC uses acl:origin to scope a grant to an application; ACP uses acp:client.
// `sparq-solid` mints a pair principal urn:sparq:pair?agent=A&client=O for those.
export const ACL_GRANTS: AclGrant[] = [
  // Public profile — anyone, any app (acl:agentClass foaf:Agent → auth:Public).
  {
    docIri: DOC_PROFILE.iri,
    system: "WAC",
    subject: "Everyone (public)",
    client: "any app",
    modes: ["acl:Read"],
    rule: `<#public> a acl:Authorization ;
  acl:accessTo  <profile/card> ;
  acl:agentClass foaf:Agent ;        # → auth:Public
  acl:mode      acl:Read .`,
  },
  // Shared calendar — the owner, and the named friend Bob, any app.
  {
    docIri: DOC_CALENDAR.iri,
    system: "WAC",
    subject: "Owner + friend Bob",
    client: "any app",
    modes: ["acl:Read"],
    rule: `<#shared> a acl:Authorization ;
  acl:accessTo <calendar/shared> ;
  acl:agent    <${OWNER}> , <${FRIEND}> ;
  acl:mode     acl:Read .`,
  },
  // Private health — ONLY the owner, ONLY from her trusted health app (ACP client scope).
  {
    docIri: DOC_HEALTH.iri,
    system: "ACP",
    subject: "Owner only",
    client: "Health Tracker app only",
    modes: ["acp:Read"],
    rule: `<#health> a acp:Policy ;
  acp:allow acp:Read ;
  acp:allOf [ a acp:Matcher ;
    acp:agent  <${OWNER}> ;
    acp:client <https://health-tracker.example/clientid> ] .`,
  },
  // Private notes — only the owner, any app.
  {
    docIri: DOC_NOTES.iri,
    system: "WAC",
    subject: "Owner only",
    client: "any app",
    modes: ["acl:Read"],
    rule: `<#notes> a acl:Authorization ;
  acl:accessTo <notes/private> ;
  acl:agent    <${OWNER}> ;
  acl:mode     acl:Read .`,
  },
];

// ----------------------------------------------------------------------------
// The (user, app) sessions — exactly `sparq-solid`'s Session { agent, client }.
// ----------------------------------------------------------------------------

export interface PodSession {
  id: string;
  /** The user (WebID), display form. */
  user: string;
  userWebId: string | null; // null = anonymous
  /** The application (client id / origin), display form. */
  app: string;
  appId: string | null; // null = no client / any
  /** Short story for the UI. */
  scenario: string;
  /**
   * The AUTHORIZED named-graph set for this (agent, client) pair — the materialized
   * `sparq-solid` decision (`accessible(...)`), computed at build time. The live
   * in-tab query is restricted to EXACTLY these graphs via FROM NAMED. Fail-closed:
   * an empty set ⇒ zero rows, indistinguishable from the Pod being empty.
   */
  authorizedGraphs: string[];
}

const HEALTH_APP = "https://health-tracker.example/clientid";
const SOCIAL_APP = "https://social.example/clientid";
const RANDOM_APP = "https://random-app.example/clientid";
const OWN_APP = "https://alice.example/dashboard/clientid";

export const SESSIONS: PodSession[] = [
  {
    id: "owner-own-app",
    user: "Alice (owner)",
    userWebId: OWNER,
    app: "Her own dashboard",
    appId: OWN_APP,
    scenario:
      "The Pod owner, signed in from her own trusted dashboard. Sees everything the dashboard is allowed — but NOT the health data, which is bound to a different, more-trusted app.",
    // owner + any app: profile (public), calendar (owner), notes (owner). NOT health
    // (that grant requires the health app's client id).
    authorizedGraphs: [DOC_PROFILE.iri, DOC_CALENDAR.iri, DOC_NOTES.iri],
  },
  {
    id: "owner-health-app",
    user: "Alice (owner)",
    userWebId: OWNER,
    app: "Health Tracker",
    appId: HEALTH_APP,
    scenario:
      "The same owner, but signed in from her trusted Health Tracker app. The (agent, client) pair now matches the ACP client-scoped grant, so the private health data becomes visible too.",
    // owner + health app: everything owner can see + the client-scoped health grant.
    authorizedGraphs: [
      DOC_PROFILE.iri,
      DOC_CALENDAR.iri,
      DOC_HEALTH.iri,
      DOC_NOTES.iri,
    ],
  },
  {
    id: "friend-social-app",
    user: "Bob (friend)",
    userWebId: FRIEND,
    app: "Social app",
    appId: SOCIAL_APP,
    scenario:
      "A friend, Bob, using a social app. The WAC grant names him on the shared calendar, so he sees the public profile + the shared calendar — but none of Alice's private data.",
    authorizedGraphs: [DOC_PROFILE.iri, DOC_CALENDAR.iri],
  },
  {
    id: "stranger-random-app",
    user: "Stranger",
    userWebId: "https://stranger.example/me#i",
    app: "Random app",
    appId: RANDOM_APP,
    scenario:
      "An unknown agent with no relationship to Alice, from an arbitrary app. Only the public profile is readable (acl:agentClass foaf:Agent). Everything else is invisible.",
    authorizedGraphs: [DOC_PROFILE.iri],
  },
  {
    id: "anonymous-no-grant",
    user: "Anonymous",
    userWebId: null,
    app: "App with no grant",
    appId: RANDOM_APP,
    scenario:
      "No authenticated WebID, and no applicable grant for this app. The fail-closed default applies: the authorized graph set is EMPTY, so the query returns nothing — indistinguishable from an empty Pod. No grant = no data.",
    // Anonymous still gets the foaf:Agent public grant on the profile.
    authorizedGraphs: [DOC_PROFILE.iri],
  },
];

/** The shared query — the SAME for every session. */
export const SHARED_QUERY = `# The SAME query for every (user, app) pair.
# sparq-solid restricts the dataset to the pair's authorized
# named graphs (FROM NAMED) before evaluation — so the rows
# that come back differ per requester.
SELECT ?graph ?s ?p ?o WHERE {
  GRAPH ?graph { ?s ?p ?o }
}
ORDER BY ?graph ?s ?p`;

/** Map a doc IRI → its doc record (for UI lookups). */
export const DOC_BY_IRI: Record<string, PodDoc> = Object.fromEntries(
  POD_DOCS.map((d) => [d.iri, d]),
);
