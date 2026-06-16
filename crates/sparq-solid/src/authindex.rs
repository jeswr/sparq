//! The session layer: read the materialized auth view (`<urn:sparq:auth>`) into a
//! transient index, expand a (WebID, client-id) session into its principal set, and
//! compute the accessible graph set per mode — `∪ allow ∖ ∪ deny`, with conditional
//! grants (ACP `noneOf`) gated by their exception matchers.
//!
//! Everything here is derivable from the auth-view TRIPLES with plain SPARQL (the
//! design doc shows the MINUS form); this index is the cached fast path (D3).

use crate::loader::graph_triples;
use crate::{AUTH_GRAPH, AUTH_NS};
use oxrdf::{NamedNode, Term};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::Graph;

/// Principal IRI matched by every session (WAC `foaf:Agent` / ACP `acp:PublicAgent`).
pub const PUBLIC: &str = "https://sparq.dev/ns/auth#Public";
/// Principal IRI matched by any session with a WebID (WAC `acl:AuthenticatedAgent` /
/// ACP `acp:AuthenticatedAgent`).
pub const AUTHENTICATED: &str = "https://sparq.dev/ns/auth#Authenticated";
/// Client IRI matched by any client in conditional-grant heads (ACP `acp:PublicClient`).
pub const ANY_CLIENT: &str = "https://sparq.dev/ns/auth#AnyClient";
/// Issuer IRI matched by any OIDC issuer (the issuer-dimension top; ACP has no "public
/// issuer" special, so an absent `acp:issuer` is issuer-unconstrained ⇒ this top).
/// [OPUS-4.8] sq-3jtd.6.
pub const ANY_ISSUER: &str = "https://sparq.dev/ns/auth#AnyIssuer";

/// A request context: who (WebID) through what (client identifier / origin) and vouched
/// for by which OIDC issuer (`acp:issuer`). `agent: None` = anonymous.
///
/// A session expands to at most 12 principals when grants are looked up ([OPUS-4.8]
/// sq-3jtd.6 — the issuer dimension doubles the pre-issuer ≤6): always `auth:Public`;
/// plus `auth:Authenticated` and the WebID itself when `agent` is given; for each of
/// those agent-principals `ap`, the minted `urn:sparq:pair?agent=…&client=…` pair when
/// `client` is given, the minted `urn:sparq:triple?agent=…&client=…&issuer=…` term when
/// `issuer` is given (with the client component being `auth:AnyClient` when no `client`),
/// and the full triple when both are given. Expansion is internal — callers just supply
/// the request context:
///
/// # Examples
///
/// ```
/// use sparq_solid::Session;
///
/// let anonymous = Session::default();
/// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None };
/// let alice_via_app = Session {
///     agent: Some("https://alice.ex/card#me"),
///     client: Some("https://app.ex"),
///     issuer: Some("https://idp.ex"),
/// };
/// assert!(anonymous.agent.is_none());
/// # let _ = (alice, alice_via_app);
/// ```
///
/// # Invariants (fail-closed)
///
/// Agent/client/issuer values inside the reserved `urn:sparq:` IRI space, or containing
/// the literal pair-encoding delimiter `&client=`, are never matched against grants: the
/// accessible set for such a session is empty (they could otherwise impersonate a
/// minted pair/triple principal — see [`AuthIndex::accessible`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct Session<'a> {
    /// The authenticated agent's WebID; `None` = anonymous.
    pub agent: Option<&'a str>,
    /// The client identifier (WAC `acl:origin` / ACP `acp:client`); `None` = any client.
    pub client: Option<&'a str>,
    /// The OIDC issuer that vouched for the WebID (ACP `acp:issuer`); `None` = any issuer.
    /// [OPUS-4.8] sq-3jtd.6 — only ACP matchers constrain on it; WAC has no issuer notion,
    /// so a WAC pod ignores it (no grant ever names a triple principal).
    pub issuer: Option<&'a str>,
}

/// An access mode, matching WAC's `acl:Read`/`Write`/`Append`/`Control` (ACP reuses
/// the same four mode IRIs by convention).
///
/// Per the WAC spec, `Control` governs the access-control document itself: the rules
/// translate `acl:Control` on a resource into `auth:read`/`auth:write` grants **on
/// its `.acl` graph**, not into access to the resource.
///
/// # Examples
///
/// ```
/// use sparq_solid::Mode;
/// // Modes key the per-session cache; they are plain copyable enum values.
/// let modes = [Mode::Read, Mode::Write, Mode::Append, Mode::Control];
/// assert_eq!(modes.len(), 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// `acl:Read` — read the resource's triples.
    Read,
    /// `acl:Write` — replace/delete the resource's triples.
    Write,
    /// `acl:Append` — add triples without removing existing ones.
    Append,
    /// `acl:Control` — read/write the resource's access-control document.
    Control,
}

impl Mode {
    fn from_pred(p: &str) -> Option<(Mode, bool)> {
        let local = p.strip_prefix(AUTH_NS)?;
        Some(match local {
            "read" => (Mode::Read, true),
            "write" => (Mode::Write, true),
            "append" => (Mode::Append, true),
            "control" => (Mode::Control, true),
            "denyRead" => (Mode::Read, false),
            "denyWrite" => (Mode::Write, false),
            "denyAppend" => (Mode::Append, false),
            "denyControl" => (Mode::Control, false),
            _ => return None,
        })
    }

    fn from_mode_iri(iri: &str) -> Option<Mode> {
        Some(match iri {
            "http://www.w3.org/ns/auth/acl#Read" => Mode::Read,
            "http://www.w3.org/ns/auth/acl#Write" => Mode::Write,
            "http://www.w3.org/ns/auth/acl#Append" => Mode::Append,
            "http://www.w3.org/ns/auth/acl#Control" => Mode::Control,
            _ => return None,
        })
    }
}

/// The deterministic pair-principal IRI minted by the rules
/// (`string:encodeForUri` + `string:concatenation` in rules/wac.n3 / rules/acp-c.n3 —
/// keep in sync; both sides share [`sparq_reason::n3::encode_for_uri`]'s RFC 3986
/// percent-encoding, so the minting is INJECTIVE — no agent/client value can smuggle
/// the `&client=` delimiter into someone else's pair. The loader's reserved-encoding
/// validation ([`Session`] values, ACL agent/origin IRIs) stays as defense in depth.)
///
/// ```
/// use sparq_solid::pair_principal;
/// assert_eq!(
///     pair_principal("https://bob.ex/card#me", "https://app.ex"),
///     "urn:sparq:pair?agent=https%3A%2F%2Fbob.ex%2Fcard%23me&client=https%3A%2F%2Fapp.ex",
/// );
/// // injective: a delimiter inside a value cannot re-bracket the pair
/// assert_ne!(pair_principal("a&client=b", "c"), pair_principal("a", "b&client=c"));
/// ```
pub fn pair_principal(agent: &str, client: &str) -> String {
    let (a, c) = (sparq_reason::n3::encode_for_uri(agent), sparq_reason::n3::encode_for_uri(client));
    format!("urn:sparq:pair?agent={a}&client={c}")
}

/// [OPUS-4.8] sq-3jtd.6 — the deterministic THREE-dimension principal IRI minted by the
/// ACP rules for an issuer-constrained grant (`string:encodeForUri` +
/// `string:concatenation` in rules/acp-c.n3 — keep in sync; the same RFC 3986
/// percent-encoding makes the minting INJECTIVE, so no agent/client/issuer value can
/// smuggle a `&client=`/`&issuer=` delimiter into another principal's term).
///
/// The `client` component is `auth:AnyClient` ([`ANY_CLIENT`]) for a matcher that
/// constrains the issuer but not the client; only an issuer-CONSTRAINED candidate mints a
/// triple — an issuer-unconstrained grant reuses the agent / [`pair_principal`] term.
///
/// ```
/// use sparq_solid::triple_principal;
/// assert_eq!(
///     triple_principal("https://bob.ex/card#me", "https://app.ex", "https://idp.ex"),
///     "urn:sparq:triple?agent=https%3A%2F%2Fbob.ex%2Fcard%23me\
///      &client=https%3A%2F%2Fapp.ex&issuer=https%3A%2F%2Fidp.ex",
/// );
/// // injective: a delimiter inside a value cannot re-bracket the triple
/// assert_ne!(
///     triple_principal("a&issuer=b", "c", "d"),
///     triple_principal("a", "c", "b&issuer=d"),
/// );
/// ```
pub fn triple_principal(agent: &str, client: &str, issuer: &str) -> String {
    let a = sparq_reason::n3::encode_for_uri(agent);
    let c = sparq_reason::n3::encode_for_uri(client);
    let i = sparq_reason::n3::encode_for_uri(issuer);
    format!("urn:sparq:triple?agent={a}&client={c}&issuer={i}")
}

#[derive(Debug, Default)]
struct ConditionalGrant {
    allow: bool,
    agent: String,  // principal-space: WebID | auth:Public | auth:Authenticated
    client: String, // auth:AnyClient | concrete client id
    // [OPUS-4.8] sq-3jtd.6: auth:AnyIssuer (issuer-unconstrained) | concrete OIDC issuer id.
    issuer: String,
    mode: Option<Mode>,
    graph: Option<NamedNode>,
    except: Vec<String>, // matcher IRIs
}

/// A transient index over the `<urn:sparq:auth>` graph's triples: principal × mode →
/// graph names, plus conditional grants (ACP `noneOf`) and the matcher accept-sets
/// they reference.
///
/// Everything the index answers is derivable from the auth-view **triples** with
/// plain SPARQL (the design doc shows the `MINUS` form); the index is the cached fast
/// path (design doc D3) and holds no state of its own — [`crate::PodStore`] rebuilds
/// it from the triples after every re-materialization. Use it directly only for
/// inspection/tests ([`crate::PodStore::auth`]) or if you manage materialization
/// yourself with the free [`crate::materialize_wac`] / [`crate::materialize_acp`]
/// functions.
///
/// # Examples
///
/// ```
/// use sparq_solid::{materialize_wac, AuthIndex, Mode, Session};
///
/// let nquads = r#"
/// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
/// "#;
/// let mut graph = sparq_core::Graph::load_dataset(nquads, "nquads")?;
/// materialize_wac(&mut graph)?;
///
/// let index = AuthIndex::from_graph(&graph);
/// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None };
/// assert_eq!(index.accessible(&alice, Mode::Read).len(), 2); // n1 + notes/
/// assert!(index.accessible(&alice, Mode::Write).is_empty()); // fail-closed
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Default)]
pub struct AuthIndex {
    /// (principal IRI, mode) → graphs, for simple allow / deny triples.
    allow: FxHashMap<(String, Mode), Vec<NamedNode>>,
    deny: FxHashMap<(String, Mode), Vec<NamedNode>>,
    cond: Vec<ConditionalGrant>,
    /// matcher IRI → accept-sets (principal space), for exceptMatcher evaluation.
    matcher_agents: FxHashMap<String, FxHashSet<String>>,
    matcher_clients: FxHashMap<String, FxHashSet<String>>,
    /// [OPUS-4.8] sq-3jtd.6 — the issuer-dimension twin of `matcher_clients`.
    matcher_issuers: FxHashMap<String, FxHashSet<String>>,
}

impl AuthIndex {
    /// Build from the dataset's `<urn:sparq:auth>` graph (empty index if absent —
    /// fail-closed).
    ///
    /// Only `auth:`-namespace triples (and the `solidx:` matcher accept-set facts the
    /// materializer copies alongside conditional grants) are interpreted; anything
    /// else in the graph is ignored. See [`AuthIndex`] for an end-to-end example.
    pub fn from_graph(graph: &Graph) -> AuthIndex {
        let mut ix = AuthIndex::default();
        let auth_name = Term::NamedNode(NamedNode::new_unchecked(AUTH_GRAPH));
        let Some((_, sub)) = graph.named.iter().find(|(n, _)| *n == auth_name) else {
            return ix;
        };
        let mut cond: FxHashMap<String, ConditionalGrant> = FxHashMap::default();
        for t in graph_triples(sub) {
            let Term::NamedNode(p) = &t[1] else { continue };
            let subj = match &t[0] {
                Term::NamedNode(n) => n.as_str().to_owned(),
                _ => continue,
            };
            if let Some((mode, is_allow)) = Mode::from_pred(p.as_str()) {
                if let Term::NamedNode(g) = &t[2] {
                    let map = if is_allow { &mut ix.allow } else { &mut ix.deny };
                    map.entry((subj, mode)).or_default().push(g.clone());
                }
                continue;
            }
            let Some(local) = p.as_str().strip_prefix(AUTH_NS) else {
                // solidx matcher accept-set facts
                match p.as_str() {
                    "https://sparq.dev/ns/solidx#acceptsAgentP" => {
                        if let Term::NamedNode(o) = &t[2] {
                            ix.matcher_agents.entry(subj).or_default().insert(o.as_str().to_owned());
                        }
                    }
                    "https://sparq.dev/ns/solidx#acceptsClientP" => {
                        if let Term::NamedNode(o) = &t[2] {
                            ix.matcher_clients.entry(subj).or_default().insert(o.as_str().to_owned());
                        }
                    }
                    // [OPUS-4.8] sq-3jtd.6: the issuer-dimension matcher accept-set fact.
                    "https://sparq.dev/ns/solidx#acceptsIssuerP" => {
                        if let Term::NamedNode(o) = &t[2] {
                            let set = ix.matcher_issuers.entry(subj).or_default();
                            set.insert(o.as_str().to_owned());
                        }
                    }
                    _ => {}
                }
                continue;
            };
            let entry = cond.entry(subj).or_default();
            match (local, &t[2]) {
                ("effect", Term::NamedNode(o)) => entry.allow = o.as_str() == format!("{AUTH_NS}Allow"),
                ("agent", Term::NamedNode(o)) => entry.agent = o.as_str().to_owned(),
                ("client", Term::NamedNode(o)) => entry.client = o.as_str().to_owned(),
                // [OPUS-4.8] sq-3jtd.6: the conditional grant's issuer head.
                ("issuer", Term::NamedNode(o)) => entry.issuer = o.as_str().to_owned(),
                ("mode", Term::NamedNode(o)) => entry.mode = Mode::from_mode_iri(o.as_str()),
                ("graph", Term::NamedNode(o)) => entry.graph = Some(o.clone()),
                ("exceptMatcher", Term::NamedNode(o)) => entry.except.push(o.as_str().to_owned()),
                _ => {}
            }
        }
        ix.cond = cond.into_values().collect();
        ix
    }

    /// The session's agent-dimension principals (most-specific last).
    fn agent_principals(s: &Session) -> Vec<String> {
        let mut ps = vec![PUBLIC.to_owned()];
        if let Some(a) = s.agent {
            ps.push(AUTHENTICATED.to_owned());
            ps.push(a.to_owned());
        }
        ps
    }

    /// All principals the session matches: the agent-dimension principals, plus, for each
    /// one, the minted `(agent, client)` pair (when a client is given), the minted
    /// `(agent, client, issuer)` triple (when an issuer is given — with `auth:AnyClient`
    /// as the client component if no client was supplied), and the full triple. [OPUS-4.8]
    /// sq-3jtd.6: ≤12 lookups (the pre-issuer ≤6 doubled by the issuer dimension).
    fn principals(s: &Session) -> Vec<String> {
        let agents = Self::agent_principals(s);
        let mut ps = agents.clone();
        if let Some(c) = s.client {
            for a in &agents {
                ps.push(pair_principal(a, c));
            }
        }
        // The issuer dimension mints a triple principal. A grant that constrains the
        // issuer but not the client mints `triple(agent, auth:AnyClient, issuer)`, so the
        // session must match that term too — hence the `auth:AnyClient` client component
        // when the session has no client.
        if let Some(i) = s.issuer {
            let client = s.client.unwrap_or(ANY_CLIENT);
            for a in &agents {
                ps.push(triple_principal(a, client, i));
                if s.client.is_some() {
                    ps.push(triple_principal(a, ANY_CLIENT, i));
                }
            }
        }
        ps
    }

    /// Does `matcher` accept this session? (Used for ACP noneOf exceptions: an
    /// accepting exception matcher suppresses the conditional grant.)
    fn matcher_accepts(&self, matcher: &str, s: &Session) -> bool {
        let agent_ok = match self.matcher_agents.get(matcher) {
            None => false, // no accept-set materialized -> cannot accept
            Some(set) => {
                set.contains(PUBLIC)
                    || s.agent
                        .map(|a| set.contains(AUTHENTICATED) || set.contains(a))
                        .unwrap_or(false)
            }
        };
        let client_ok = match self.matcher_clients.get(matcher) {
            None => false,
            Some(set) => {
                set.contains(ANY_CLIENT) || s.client.map(|c| set.contains(c)).unwrap_or(false)
            }
        };
        // [OPUS-4.8] sq-3jtd.6: the issuer-dimension twin of the client check.
        let issuer_ok = match self.matcher_issuers.get(matcher) {
            None => false,
            Some(set) => {
                set.contains(ANY_ISSUER) || s.issuer.map(|i| set.contains(i)).unwrap_or(false)
            }
        };
        agent_ok && client_ok && issuer_ok
    }

    /// Does a conditional grant's (agent, client, issuer) head apply to this session?
    /// [OPUS-4.8] sq-3jtd.6: the issuer head is the twin of the client head — the grant's
    /// `auth:AnyIssuer` matches any session, else the session's issuer must equal it.
    fn cond_applies(&self, g: &ConditionalGrant, s: &Session) -> bool {
        let agent_ok = Self::agent_principals(s).contains(&g.agent);
        let client_ok = g.client == ANY_CLIENT || s.client == Some(g.client.as_str());
        let issuer_ok = g.issuer == ANY_ISSUER || s.issuer == Some(g.issuer.as_str());
        agent_ok && client_ok && issuer_ok && !g.except.iter().any(|m| self.matcher_accepts(m, s))
    }

    /// The sorted, deduplicated graph set this session may access in `mode`:
    /// `∪ allow(principals) ∖ ∪ deny(principals)` (deny-overrides across principals),
    /// with conditional grants/denies (ACP `noneOf`) applied when their
    /// (agent, client, issuer) head matches the session and no exception matcher accepts it.
    ///
    /// Never panics and never errors — every unauthorized condition degrades to the
    /// **empty set**:
    ///
    /// - no auth view materialized → empty index → empty set;
    /// - no grant names any of the session's principals → empty set;
    /// - session values inside the reserved principal encoding fail CLOSED — a
    ///   caller-supplied "WebID" like `urn:sparq:pair?agent=…&client=…` must not
    ///   impersonate a minted pair principal (roborev 1727).
    ///
    /// Prefer [`crate::PodStore::accessible`], which memoizes this walk per
    /// (agent, client, issuer, mode) until the next re-materialization.
    pub fn accessible(&self, s: &Session, mode: Mode) -> Vec<NamedNode> {
        let invalid = |v: Option<&str>| v.is_some_and(|x| !crate::loader::session_value_allowed(x));
        // [OPUS-4.8] sq-3jtd.6: the issuer is a triple-principal ingredient too — a session
        // issuer inside the reserved space could otherwise impersonate a minted triple.
        if invalid(s.agent) || invalid(s.client) || invalid(s.issuer) {
            return Vec::new();
        }
        let principals = Self::principals(s);
        let mut allowed: FxHashSet<NamedNode> = FxHashSet::default();
        let mut denied: FxHashSet<&NamedNode> = FxHashSet::default();
        for p in &principals {
            if let Some(gs) = self.allow.get(&(p.clone(), mode)) {
                allowed.extend(gs.iter().cloned());
            }
            if let Some(gs) = self.deny.get(&(p.clone(), mode)) {
                denied.extend(gs.iter());
            }
        }
        for c in &self.cond {
            if c.mode == Some(mode) && self.cond_applies(c, s) {
                if let Some(g) = &c.graph {
                    if c.allow {
                        allowed.insert(g.clone());
                    } else {
                        denied.insert(g);
                    }
                }
            }
        }
        let denied: FxHashSet<NamedNode> = denied.into_iter().cloned().collect();
        let mut out: Vec<NamedNode> = allowed.into_iter().filter(|g| !denied.contains(g)).collect();
        out.sort_unstable_by(|a, b| a.as_str().cmp(b.as_str()));
        out
    }
}
