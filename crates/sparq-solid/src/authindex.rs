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

pub const PUBLIC: &str = "https://sparq.dev/ns/auth#Public";
pub const AUTHENTICATED: &str = "https://sparq.dev/ns/auth#Authenticated";
pub const ANY_CLIENT: &str = "https://sparq.dev/ns/auth#AnyClient";

/// A request context: who (WebID) through what (client identifier / origin).
/// `agent: None` = anonymous.
#[derive(Debug, Clone, Copy, Default)]
pub struct Session<'a> {
    pub agent: Option<&'a str>,
    pub client: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Read,
    Write,
    Append,
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
/// (`string:concatenation` in rules/wac.n3 / rules/acp-c.n3 — keep in sync).
pub fn pair_principal(agent: &str, client: &str) -> String {
    format!("urn:sparq:pair?agent={agent}&client={client}")
}

#[derive(Debug, Default)]
struct ConditionalGrant {
    allow: bool,
    agent: String,  // principal-space: WebID | auth:Public | auth:Authenticated
    client: String, // auth:AnyClient | concrete client id
    mode: Option<Mode>,
    graph: Option<NamedNode>,
    except: Vec<String>, // matcher IRIs
}

#[derive(Debug, Default)]
pub struct AuthIndex {
    /// (principal IRI, mode) → graphs, for simple allow / deny triples.
    allow: FxHashMap<(String, Mode), Vec<NamedNode>>,
    deny: FxHashMap<(String, Mode), Vec<NamedNode>>,
    cond: Vec<ConditionalGrant>,
    /// matcher IRI → accept-sets (principal space), for exceptMatcher evaluation.
    matcher_agents: FxHashMap<String, FxHashSet<String>>,
    matcher_clients: FxHashMap<String, FxHashSet<String>>,
}

impl AuthIndex {
    /// Build from the dataset's `<urn:sparq:auth>` graph (empty index if absent —
    /// fail-closed).
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
                    _ => {}
                }
                continue;
            };
            let entry = cond.entry(subj).or_default();
            match (local, &t[2]) {
                ("effect", Term::NamedNode(o)) => entry.allow = o.as_str() == format!("{AUTH_NS}Allow"),
                ("agent", Term::NamedNode(o)) => entry.agent = o.as_str().to_owned(),
                ("client", Term::NamedNode(o)) => entry.client = o.as_str().to_owned(),
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

    /// All principals the session matches (agent-dimension + (agent, client) pairs).
    fn principals(s: &Session) -> Vec<String> {
        let mut ps = Self::agent_principals(s);
        if let Some(c) = s.client {
            for a in Self::agent_principals(s) {
                ps.push(pair_principal(&a, c));
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
        agent_ok && client_ok
    }

    /// Does a conditional grant's (agent, client) head apply to this session?
    fn cond_applies(&self, g: &ConditionalGrant, s: &Session) -> bool {
        let agent_ok = Self::agent_principals(s).iter().any(|p| *p == g.agent);
        let client_ok = g.client == ANY_CLIENT || s.client == Some(g.client.as_str());
        agent_ok && client_ok && !g.except.iter().any(|m| self.matcher_accepts(m, s))
    }

    /// The sorted, deduplicated graph set this session may access in `mode`:
    /// `∪ allow(principals) ∖ ∪ deny(principals)` (deny-overrides across principals).
    pub fn accessible(&self, s: &Session, mode: Mode) -> Vec<NamedNode> {
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
