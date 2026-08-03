//! The small, fixed, versioned keyword layer (lever 1 — design §3.1, Phase 3, sq-vfeme).
//!
//! A *bare keyword* like `K:type` or `K:derivedFrom` expands **pre-parse** to the canonical
//! `<iri>` of a high-frequency PKG predicate/class, so an agent need not emit + get-right a
//! `PREFIX` line or a long property IRI. The set is **tiny, frozen and versioned** — the
//! "constrained-DSL" shape the literature says *helps* (design §1.4: small + fixed removes
//! degrees of freedom, rather than a sprawling alias dictionary that *adds* novelty).
//!
//! This is the lean half of the surface: it depends on **nothing** but this static table and
//! the existing `spargebra` canary — no `sparq-core`, no model, no network — so it lives in
//! the **default build** (it is *not* behind the `vectors` feature that lever 3's `V()`
//! needs). Lever 1 (cheap, no-model keyword expansion) and lever 3 (`V()` concept resolution)
//! are independent surfaces.
//!
//! ## The `K:` sigil (the one fixed token form)
//!
//! A keyword is written `K:<name>` — an explicit, unmistakable sigil, *not* a bare word.
//! A bare-word scan over arbitrary SPARQL is far too collision-prone (every variable suffix
//! or prefixed-name local part could match); the `K:` sigil makes a keyword a single, fixed,
//! example-anchored token that can never be confused with an ordinary identifier. Everything
//! that is not a `K:<name>` token passes through as ordinary SPARQL (prefixed names still
//! work).
//!
//! ## The two hard-error guardrails (design §3.1 (i)/(iii))
//!
//! 1. **Collision with a real prefixed name is a HARD error, never a silent guess.** If the
//!    source declares `PREFIX K:` (a real prefix literally named `K`), then `K:type` is
//!    genuinely ambiguous between *keyword expansion* and *prefix expansion* — we refuse with
//!    [`crate::TerseError::KeywordPrefixCollision`] rather than guess which the agent meant.
//! 2. **An unknown keyword is a HARD error.** `K:<name>` where `<name>` is not in the frozen
//!    legend is [`crate::TerseError::UnknownKeyword`] (carrying the legend version + the close
//!    suggestions) — the surface never invents an expansion for a keyword it does not know.
//!
//! Every expansion is **echoed** in [`crate::Expansion::keywords`] (and lands verbatim in
//! `canonical_sparql`), so the agent audits exactly what each `K:<name>` became (design
//! §3.1 (ii) — "the expansion is always echoed").

/// The verifiable record of one `K:<name>` keyword expansion (lever 1, design §3.1).
/// Returned in [`crate::Expansion::keywords`] so the agent audits exactly what each keyword
/// became: the keyword written, the absolute IRI it expanded to, and the frozen legend
/// version it was resolved against. Every expansion is **always echoed** (design §3.1 (ii)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordExpansion {
    /// The keyword name written after `K:` (e.g. `derivedFrom`).
    pub keyword: String,
    /// The absolute IRI it expanded to (e.g. `http://www.w3.org/ns/prov#wasDerivedFrom`),
    /// which is spliced verbatim (as `<iri>`) into [`crate::Expansion::canonical_sparql`].
    pub iri: String,
    /// The frozen legend version this expansion was resolved against (design §3.1 — "fixed
    /// and versioned").
    pub legend_version: String,
}

/// The sigil that marks a keyword token: `K:<name>`.
pub(crate) const SIGIL: &str = "K";

/// The frozen keyword-legend version. Bumped only by a deliberate, maintainer-approved change
/// to the frozen legend; surfaced in [`crate::KeywordExpansion`] and the unknown-keyword error so a
/// caller can pin / publish the exact set it expanded against (design §3.1 — "fixed and
/// versioned"). This is **v1** of the frozen set; broad adoption is gated on the Phase-5 A/B
/// measuring a real cache-discounted win (design §5, a future bead).
pub const LEGEND_VERSION: &str = "pkg-keywords/v1";

// The namespaces the legend abbreviates (from crates/sparq-kb/ontology/pkg/pkg.ttl).
const PKG: &str = "https://sparq.dev/ns/pkg#";
const PROV: &str = "http://www.w3.org/ns/prov#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";
const DCTERMS: &str = "http://purl.org/dc/terms/";
const CITO: &str = "http://purl.org/spar/cito/";
const SECX: &str = "https://w3id.org/zkp-sparql/sec-prop#";

/// One frozen legend entry: a keyword *name* (the part after `K:`), the namespace it lives in,
/// and the local name within that namespace. The absolute IRI is `namespace + local` — kept
/// as `(namespace, local)` rather than a pre-joined literal so the table is readable and the
/// shared namespace constants cannot drift between entries.
struct Entry {
    /// The keyword name written after `K:` (case-sensitive).
    name: &'static str,
    /// The absolute namespace IRI.
    namespace: &'static str,
    /// The local name within `namespace`.
    local: &'static str,
}

const fn e(name: &'static str, namespace: &'static str, local: &'static str) -> Entry {
    Entry {
        name,
        namespace,
        local,
    }
}

/// The small, fixed keyword legend (design §3.1, scoped to the PKG ontology's actual hot
/// predicates/classes from `crates/sparq-kb/ontology/pkg/pkg.ttl` +
/// `research/dogfooding-sparq-knowledge-graph.md` §2). Frozen at [`LEGEND_VERSION`]; NOT a
/// general alias dictionary — only the highest-frequency PKG terms an agent writes by hand.
/// Names are case-sensitive and never overlap (asserted in a unit test). Verbose API detail
/// lives in rustdoc / the README, not duplicated here.
const LEGEND_ENTRIES: &[Entry] = &[
    // --- RDF/RDFS spine (the universally-known shape; `type` mirrors Turtle's `a`) ---
    e("type", RDF, "type"),
    e("label", RDFS, "label"),
    e("subClassOf", RDFS, "subClassOf"),
    e("subPropertyOf", RDFS, "subPropertyOf"),
    e("comment", RDFS, "comment"),
    // --- PROV provenance (the verbose, error-prone-to-type property IRIs) ---
    e("derivedFrom", PROV, "wasDerivedFrom"),
    e("generatedBy", PROV, "wasGeneratedBy"),
    e("attributedTo", PROV, "wasAttributedTo"),
    e("generatedAtTime", PROV, "generatedAtTime"),
    // --- SKOS labelling / scheme membership ---
    e("prefLabel", SKOS, "prefLabel"),
    e("altLabel", SKOS, "altLabel"),
    e("broader", SKOS, "broader"),
    e("narrower", SKOS, "narrower"),
    e("related", SKOS, "related"),
    e("inScheme", SKOS, "inScheme"),
    // --- DC terms ---
    e("subject", DCTERMS, "subject"),
    e("isPartOf", DCTERMS, "isPartOf"),
    e("replaces", DCTERMS, "replaces"),
    // --- CiTO citation/argument edges ---
    e("supports", CITO, "supports"),
    e("disagreesWith", CITO, "disagreesWith"),
    e("citesAsEvidence", CITO, "citesAsEvidence"),
    // --- PKG net-new hot predicates/classes ---
    e("about", PKG, "about"),
    e("confidence", PKG, "confidence"),
    e("assurance", PKG, "assurance"),
    e("verdict", PKG, "verdict"),
    e("dependsOn", PKG, "dependsOn"),
    e("blockedBy", PKG, "blockedBy"),
    e("status", PKG, "status"),
    e("issueType", PKG, "issueType"),
    e("priority", PKG, "priority"),
    e("surface", PKG, "surface"),
    e("discoveredFrom", PKG, "discoveredFrom"),
    e("implementedBy", PKG, "implementedBy"),
    e("couldBeMergedWith", PKG, "couldBeMergedWith"),
    e("exploredStatus", PKG, "exploredStatus"),
    e("followUpPriority", PKG, "followUpPriority"),
    e("Source", PKG, "Source"),
    e("Finding", PKG, "Finding"),
    e("Task", PKG, "Task"),
    e("Technique", PKG, "Technique"),
    // --- The (orthogonal) assurance axis (maintainer's sec-prop: vocabulary) ---
    e("Proven", SECX, "Proven"),
    e("Claimed", SECX, "Claimed"),
    e("Conjectured", SECX, "Conjectured"),
];

/// Looks up a keyword `name` (the part after `K:`) in the frozen legend, returning its
/// absolute IRI string. `None` if the name is not in the legend (the caller turns that into a
/// loud [`crate::TerseError::UnknownKeyword`], never a silent guess).
pub(crate) fn lookup(name: &str) -> Option<String> {
    LEGEND_ENTRIES
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| format!("{}{}", entry.namespace, entry.local))
}

/// The number of entries in the frozen legend (for the in-context legend card and tests).
pub fn legend_len() -> usize {
    LEGEND_ENTRIES.len()
}

/// Returns the frozen legend as `(keyword, absolute_iri)` pairs in declaration order — the
/// material an agent publishes as an **in-context legend card** behind the prompt-cache
/// breakpoint (design §3.1 — "publish the legend as an in-context card"; §1.6 — the token win
/// is a *caching* property, so the legend must sit behind the cache breakpoint, not be
/// re-billed per turn). Pairs are stable across a [`LEGEND_VERSION`].
pub fn legend() -> Vec<(&'static str, String)> {
    LEGEND_ENTRIES
        .iter()
        .map(|entry| (entry.name, format!("{}{}", entry.namespace, entry.local)))
        .collect()
}

/// Renders the frozen legend as a compact, copy-pasteable in-context card: a `LEGEND_VERSION`
/// header line, then one `K:<name> -> <iri>` line per entry. This is the artefact a caller
/// places **once, behind the prompt-cache breakpoint**, so the keyword set is anchored by an
/// in-context example table (the §1.4 precondition for a terse dialect to help) at ~0.1× warm
/// cost (design §1.6).
pub fn legend_card() -> String {
    let mut out = String::new();
    out.push_str("# sparq-terse keyword legend ");
    out.push_str(LEGEND_VERSION);
    out.push_str("\n# usage: write K:<name> in any term position; it expands to <iri>.\n");
    for entry in LEGEND_ENTRIES {
        out.push_str(SIGIL);
        out.push(':');
        out.push_str(entry.name);
        out.push_str(" -> <");
        out.push_str(entry.namespace);
        out.push_str(entry.local);
        out.push_str(">\n");
    }
    out
}

/// The up-to-`limit` legend names closest to `name` by case-insensitive prefix / substring
/// overlap, best first — the "did-you-mean" candidates carried by
/// [`crate::TerseError::UnknownKeyword`] so a mistyped keyword is a *recoverable* signal, not
/// a dead end. This is purely a diagnostic over the frozen names; it never *applies* a
/// suggestion (that would be the lever-2 silent-rewrite anti-pattern; design §3.2).
pub(crate) fn suggestions(name: &str, limit: usize) -> Vec<&'static str> {
    let needle = name.to_ascii_lowercase();
    let mut scored: Vec<(u32, &'static str)> = LEGEND_ENTRIES
        .iter()
        .filter_map(|entry| {
            let cand = entry.name.to_ascii_lowercase();
            let score = name_overlap(&needle, &cand);
            (score > 0).then_some((score, entry.name))
        })
        .collect();
    // Best score first; ties broken by name for a stable, deterministic order.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    scored.truncate(limit);
    scored.into_iter().map(|(_, n)| n).collect()
}

/// A tiny lowercase-overlap score for "did-you-mean": an exact match scores highest, then a
/// shared prefix (length-weighted), then a substring containment. Deterministic and
/// dependency-free — it only orders the *candidate list* the agent disambiguates against.
fn name_overlap(needle: &str, cand: &str) -> u32 {
    if needle == cand {
        return 1_000;
    }
    let shared_prefix = needle
        .bytes()
        .zip(cand.bytes())
        .take_while(|(a, b)| a == b)
        .count() as u32;
    if cand.contains(needle) || needle.contains(cand) {
        return 100 + shared_prefix;
    }
    shared_prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legend_names_are_unique_and_iris_valid() {
        // The frozen set must have no duplicate keyword and every expansion must be a
        // well-formed absolute IRI (so a bare K:<name> can never produce non-canonical
        // SPARQL — the load-bearing invariant for the keyword layer).
        let mut seen = std::collections::HashSet::new();
        for (name, iri) in legend() {
            assert!(
                seen.insert(name),
                "duplicate keyword in the frozen legend: {}",
                name
            );
            assert!(
                oxiri_ok(&iri),
                "legend IRI for K:{} is not a valid absolute IRI: {}",
                name,
                iri
            );
        }
        assert_eq!(seen.len(), legend_len());
    }

    /// A minimal absolute-IRI check that does not pull a dependency into the lean default
    /// build: scheme `://` and no whitespace. The spargebra canary is the real conformance
    /// gate downstream; this only catches a malformed legend literal early.
    fn oxiri_ok(iri: &str) -> bool {
        iri.contains("://") && !iri.chars().any(|c| c.is_whitespace())
    }

    #[test]
    fn lookup_hits_and_misses() {
        assert_eq!(
            lookup("type").as_deref(),
            Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
        );
        assert_eq!(
            lookup("derivedFrom").as_deref(),
            Some("http://www.w3.org/ns/prov#wasDerivedFrom")
        );
        assert_eq!(lookup("not_a_keyword"), None);
        // Case-sensitive: `Type` is not `type`.
        assert_eq!(lookup("Type"), None);
    }

    #[test]
    fn suggestions_are_close_and_ordered() {
        // A near-miss surfaces the intended keyword first.
        let s = suggestions("derived", 3);
        assert!(s.contains(&"derivedFrom"), "got {:?}", s);
        // An exact name ranks itself top.
        assert_eq!(suggestions("label", 1), vec!["label"]);
        // A wholly-unrelated token may yield nothing — that is fine (still a loud error).
        assert!(suggestions("zzzzzz", 3).is_empty());
    }

    #[test]
    fn legend_card_lists_every_entry() {
        let card = legend_card();
        assert!(card.contains(LEGEND_VERSION));
        for (name, iri) in legend() {
            assert!(
                card.contains(&format!("K:{} -> <{}>", name, iri)),
                "missing {}",
                name
            );
        }
    }
}
