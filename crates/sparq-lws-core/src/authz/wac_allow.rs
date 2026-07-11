// AUTHORED-BY Claude Opus 4.8
//! The Solid Protocol `WAC-Allow` response-header serialiser.
//!
//! Format: `user="<modes>",public="<modes>"` — each modes list is space-separated in canonical
//! (read/write/append/control) order. An audience with NO modes still appears, with an empty quoted
//! string (`user=""`) — the conformance harness's `parseWacAllowHeader` expects BOTH keys present.
//!
//! Examples: `user="read write control",public="read"` · `user="",public=""`.
//!
//! Ported (semantics) from prod-solid-server `src/authz/wacAllow.ts`.

use std::collections::BTreeSet;

use super::mode::AccessMode;

/// The effective access modes over a resource, split by audience: `user` is what the requester may
/// do; `public` is what an unauthenticated agent may do. Both are computed from the SAME effective
/// ACL, so the advertisement cannot diverge from the gate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectivePermissions {
    pub user: BTreeSet<AccessMode>,
    pub public: BTreeSet<AccessMode>,
}

/// Canonical mode ordering inside a `WAC-Allow` group.
const CANONICAL_ORDER: [AccessMode; 4] = [
    AccessMode::Read,
    AccessMode::Write,
    AccessMode::Append,
    AccessMode::Control,
];

fn serialise_modes(modes: &BTreeSet<AccessMode>) -> String {
    CANONICAL_ORDER
        .iter()
        .filter(|m| modes.contains(m))
        .map(|m| m.token())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Serialise an [`EffectivePermissions`] pair as the `WAC-Allow` header value.
pub fn wac_allow_header(perms: &EffectivePermissions) -> String {
    format!(
        "user=\"{}\",public=\"{}\"",
        serialise_modes(&perms.user),
        serialise_modes(&perms.public)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(modes: &[AccessMode]) -> BTreeSet<AccessMode> {
        modes.iter().copied().collect()
    }

    #[test]
    fn full_user_some_public() {
        let perms = EffectivePermissions {
            user: set(&[AccessMode::Read, AccessMode::Write, AccessMode::Control]),
            public: set(&[AccessMode::Read]),
        };
        assert_eq!(
            wac_allow_header(&perms),
            "user=\"read write control\",public=\"read\""
        );
    }

    #[test]
    fn empty_audiences_still_emit_both_keys() {
        let perms = EffectivePermissions::default();
        assert_eq!(wac_allow_header(&perms), "user=\"\",public=\"\"");
    }

    #[test]
    fn canonical_order_is_stable() {
        // Insert out of order; output is read/write/append/control.
        let perms = EffectivePermissions {
            user: set(&[
                AccessMode::Control,
                AccessMode::Append,
                AccessMode::Read,
                AccessMode::Write,
            ]),
            public: BTreeSet::new(),
        };
        assert_eq!(
            wac_allow_header(&perms),
            "user=\"read write append control\",public=\"\""
        );
    }

    #[test]
    fn append_only_is_not_expanded_to_write() {
        // The header reflects EXACTLY the granted modes — an `append` grant is NOT rendered as `write`
        // (the conformance `read/append` `only` check requires the literal set).
        let perms = EffectivePermissions {
            user: set(&[AccessMode::Read, AccessMode::Append]),
            public: set(&[AccessMode::Read, AccessMode::Append]),
        };
        assert_eq!(
            wac_allow_header(&perms),
            "user=\"read append\",public=\"read append\""
        );
    }
}
