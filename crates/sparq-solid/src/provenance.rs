//! [OPUS-4.8] sq-3jtd.5 — TRUSTED per-resource creator/owner provenance.
//!
//! `acp:CreatorAgent` / `acp:OwnerAgent` matchers grant the *creator* / *owner* of the
//! resource being accessed. "Who created `<r>`" is structural storage metadata the
//! storage layer (PSS) knows when it mints the resource — it is **not** in the pod or the
//! ACR. This type is the TRUSTED CHANNEL by which that caller supplies the facts.
//!
//! # Security boundary (design doc §2.4)
//!
//! The creator/owner fact MUST arrive here, from the trusted caller — it is **never**
//! read from the resource graph. A writer who can put `<r> solidx:creator <self>` in a
//! document they control must not thereby grant themselves access; the loader only ever
//! synthesizes `solidx:creator`/`solidx:owner` facts from THIS map, never from pod
//! content. Supplying a value through this API is an assertion of trust by the caller.

use rustc_hash::FxHashMap;

/// The trusted creator and owner WebID of one resource. `None` = unknown to the caller
/// (so no `CreatorAgent`/`OwnerAgent` grant can ever fire on that resource — fail-closed).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResourceProvenance {
    creator: Option<String>,
    owner: Option<String>,
}

/// Caller-supplied (TRUSTED) per-resource creator/owner WebIDs, the input to
/// `acp:CreatorAgent` / `acp:OwnerAgent` matcher resolution. Built by the storage layer
/// and passed to [`crate::PodStore::materialize_acp_with`] /
/// [`crate::materialize_acp_with`]; an empty map (the default) means no creator/owner is
/// known, so no `CreatorAgent`/`OwnerAgent` matcher ever grants — fully fail-closed.
///
/// These facts are asserted by the caller and are **never** read from the resource graph —
/// see the type-level docs above for the full trust boundary.
///
/// # Examples
///
/// ```
/// use sparq_solid::AccessProvenance;
///
/// let mut prov = AccessProvenance::default();
/// prov.set_creator("https://pod.ex/notes/n1.ttl", "https://alice.ex/card#me");
/// prov.set_owner("https://pod.ex/notes/n1.ttl", "https://alice.ex/card#me");
/// assert_eq!(prov.creator("https://pod.ex/notes/n1.ttl"), Some("https://alice.ex/card#me"));
/// assert_eq!(prov.creator("https://pod.ex/notes/n2.ttl"), None); // unknown -> fail-closed
/// ```
#[derive(Debug, Clone, Default)]
pub struct AccessProvenance {
    by_resource: FxHashMap<String, ResourceProvenance>,
}

impl AccessProvenance {
    /// An empty provenance map (no creator/owner known for any resource).
    pub fn new() -> AccessProvenance {
        AccessProvenance::default()
    }

    /// Assert (TRUSTED) that `creator` is the creator WebID of `resource`. Replaces any
    /// previous creator for that resource. The value is taken on trust from the caller —
    /// see the [`AccessProvenance`] type docs for the security boundary.
    pub fn set_creator(&mut self, resource: impl Into<String>, creator: impl Into<String>) {
        self.by_resource.entry(resource.into()).or_default().creator = Some(creator.into());
    }

    /// Assert (TRUSTED) that `owner` is the owner WebID of `resource`. Replaces any
    /// previous owner for that resource.
    pub fn set_owner(&mut self, resource: impl Into<String>, owner: impl Into<String>) {
        self.by_resource.entry(resource.into()).or_default().owner = Some(owner.into());
    }

    /// The trusted creator WebID of `resource`, if the caller supplied one.
    pub fn creator(&self, resource: &str) -> Option<&str> {
        self.by_resource.get(resource).and_then(|p| p.creator.as_deref())
    }

    /// The trusted owner WebID of `resource`, if the caller supplied one.
    pub fn owner(&self, resource: &str) -> Option<&str> {
        self.by_resource.get(resource).and_then(|p| p.owner.as_deref())
    }

    /// Whether any creator or owner fact was supplied (an empty map materializes no
    /// `CreatorAgent`/`OwnerAgent` grants).
    pub fn is_empty(&self) -> bool {
        self.by_resource.is_empty()
    }

    /// Iterate `(resource, creator?, owner?)` for every resource with a fact. Internal —
    /// used by the loader to synthesize `solidx:creator`/`solidx:owner` reasoning facts.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, Option<&str>, Option<&str>)> {
        self.by_resource
            .iter()
            .map(|(r, p)| (r.as_str(), p.creator.as_deref(), p.owner.as_deref()))
    }
}
