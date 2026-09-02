//! The transpiler's error type. Every failure is *loud*: the surface never silently
//! accepts wrong input or hides a rewrite (design §6 — "opt-in, verifiable, loud-failing").

use std::fmt;

/// A transpile / concept-resolution failure. Each variant is a deliberate loud failure,
/// not a silent fallback: a `V()` that cannot be confidently resolved is an error the agent
/// sees, never a guess (design §6.3, "loud-fail beats silent-wrong").
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TerseError {
    /// The input used a `V("phrase")` construct but the resolution capability is
    /// unavailable — either the `vectors` feature is off, or no resolution context was
    /// supplied to this entry point. Reported (not silently passed through) so the agent
    /// learns the construct needs the opt-in surface, rather than getting a baffling
    /// downstream parse error.
    FeatureRequired {
        /// The phrase from the `V("phrase")` that could not be resolved.
        phrase: String,
        /// Why resolution was unavailable.
        why: String,
    },
    /// The canonical SPARQL the transpiler was about to emit does not parse under the
    /// vendored `spargebra` parser — the silent-rewrite canary (design §6.7). Surfaced
    /// rather than handing the agent invalid SPARQL.
    CanaryFailed {
        /// The canonical SPARQL that failed to parse.
        sparql: String,
        /// The underlying `spargebra` parse error.
        parse_error: String,
    },
    /// A `V("phrase")` could not be resolved confidently and so was **not** bound (design
    /// §6.3, confidence-gated): the score was below the floor, or the top candidate and its
    /// runner-up were within the ambiguity margin. Carries the candidate list so the agent
    /// can disambiguate — it is a `needs-disambiguation` signal, never a silent guess.
    Unresolved {
        /// The phrase that could not be confidently resolved.
        phrase: String,
        /// Human-readable reason (below-floor / ambiguous / no candidates).
        why: String,
        /// The candidate IRIs and their scores, best first (may be empty).
        candidates: Vec<(String, f32)>,
    },
    /// The vector store used for the `V()` fallback was built against a different graph
    /// generation than the graph being queried — the mandatory staleness guard (design
    /// §6.5). A stale store would silently mis-resolve, so this is a hard error.
    StaleStore {
        /// The phrase whose resolution hit the stale store.
        phrase: String,
        /// The underlying staleness-check message.
        detail: String,
    },
    /// A `K:<name>` keyword (lever 1, design §3.1) named a keyword that is **not** in the
    /// frozen legend — surfaced loudly with the legend version + the close suggestions, rather
    /// than inventing an expansion (design §3.1 — a keyword is never a silent guess). The
    /// suggestions are a *diagnostic only*; they are never auto-applied.
    UnknownKeyword {
        /// The unknown keyword name (the part after `K:`).
        keyword: String,
        /// The frozen legend version against which it was unknown.
        legend_version: String,
        /// The closest legend names (did-you-mean), best first; may be empty.
        suggestions: Vec<String>,
    },
    /// A `K:<name>` keyword (lever 1, design §3.1) collided with a **real** prefix literally
    /// named `K` declared in the query (`PREFIX K:` / `PREFIX K: <...>`). The token is then
    /// genuinely ambiguous between *keyword expansion* and *prefix expansion*, so the surface
    /// refuses rather than guessing which the author meant (design §3.1 (iii) — "a keyword
    /// that collides with a real prefixed name is a hard error, never a silent guess").
    KeywordPrefixCollision {
        /// The `K:<name>` keyword token whose expansion is ambiguous.
        keyword: String,
    },
}

impl fmt::Display for TerseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Positional format args throughout (avoids the CodeQL rust/unused-variable false
        // positive on inline-captured identifiers — see the SPARQ CodeQL note).
        match self {
            TerseError::FeatureRequired { phrase, why } => {
                write!(f, "cannot resolve V(\"{}\"): {}", phrase, why)
            }
            TerseError::CanaryFailed {
                sparql,
                parse_error,
            } => write!(
                f,
                "silent-rewrite canary failed: emitted SPARQL does not parse ({}): {}",
                parse_error, sparql
            ),
            TerseError::Unresolved {
                phrase,
                why,
                candidates,
            } => {
                write!(f, "V(\"{}\") not bound ({}); candidates: [", phrase, why)?;
                for (i, (iri, score)) in candidates.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} ({:.3})", iri, score)?;
                }
                write!(f, "]")
            }
            TerseError::StaleStore { phrase, detail } => write!(
                f,
                "V(\"{}\") aborted: vector store is stale vs the graph ({})",
                phrase, detail
            ),
            TerseError::UnknownKeyword {
                keyword,
                legend_version,
                suggestions,
            } => {
                write!(
                    f,
                    "unknown keyword K:{} (not in the frozen legend {})",
                    keyword, legend_version
                )?;
                if !suggestions.is_empty() {
                    write!(f, "; did you mean ")?;
                    for (i, s) in suggestions.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "K:{}", s)?;
                    }
                    write!(f, "?")?;
                }
                Ok(())
            }
            TerseError::KeywordPrefixCollision { keyword } => write!(
                f,
                "keyword K:{} collides with a real `PREFIX K:` declared in the query; rename \
                 the prefix or use the full IRI — never silently guessed",
                keyword
            ),
        }
    }
}

impl std::error::Error for TerseError {}
