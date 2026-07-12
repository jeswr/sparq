//! Structured differences between two mined effective schemas.

use crate::Introspection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One class or predicate whose observed count differs between two schemas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffEntry {
    /// The class or predicate IRI.
    pub iri: String,
    /// Its observed count in the base schema, or zero when it was added.
    pub base: u64,
    /// Its observed count in the new schema, or zero when it was removed.
    pub new: u64,
}

/// A deterministic schema-drift report between two introspection results.
///
/// Every entry vector is sorted by ascending IRI. Class counts are mined class
/// instances; predicate counts are predicate triples.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDiff {
    /// Total triples in the base schema's dataset.
    pub triples_base: u64,
    /// Total triples in the new schema's dataset.
    pub triples_new: u64,
    /// Classes present only in the new schema.
    pub added_classes: Vec<DiffEntry>,
    /// Classes present only in the base schema.
    pub removed_classes: Vec<DiffEntry>,
    /// Classes present in both schemas whose instance counts differ.
    pub changed_classes: Vec<DiffEntry>,
    /// Predicates present only in the new schema.
    pub added_predicates: Vec<DiffEntry>,
    /// Predicates present only in the base schema.
    pub removed_predicates: Vec<DiffEntry>,
    /// Predicates present in both schemas whose triple counts differ.
    pub changed_predicates: Vec<DiffEntry>,
}

impl SchemaDiff {
    /// Serialises this report as pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("schema diff serialises to JSON")
    }
}

type EntryGroups = (Vec<DiffEntry>, Vec<DiffEntry>, Vec<DiffEntry>);

fn diff_counts<'base, 'new>(
    base_counts: impl IntoIterator<Item = (&'base str, u64)>,
    new_counts: impl IntoIterator<Item = (&'new str, u64)>,
) -> EntryGroups {
    let base: BTreeMap<&str, u64> = base_counts.into_iter().collect();
    let new: BTreeMap<&str, u64> = new_counts.into_iter().collect();
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (&iri, &base_count) in &base {
        match new.get(iri) {
            None => removed.push(DiffEntry {
                iri: iri.to_owned(),
                base: base_count,
                new: 0,
            }),
            Some(&new_count) if base_count != new_count => changed.push(DiffEntry {
                iri: iri.to_owned(),
                base: base_count,
                new: new_count,
            }),
            Some(_) => {}
        }
    }

    for (&iri, &new_count) in &new {
        if !base.contains_key(iri) {
            added.push(DiffEntry {
                iri: iri.to_owned(),
                base: 0,
                new: new_count,
            });
        }
    }

    (added, removed, changed)
}

impl Introspection {
    /// Reports class, predicate, and dataset-size drift from `self` to `other`.
    ///
    /// Classes are compared by instance count and predicates by triple count.
    /// Unchanged entries are omitted.
    pub fn diff(&self, other: &Introspection) -> SchemaDiff {
        let (added_classes, removed_classes, changed_classes) = diff_counts(
            self.classes
                .iter()
                .map(|profile| (profile.class.as_str(), profile.instances)),
            other
                .classes
                .iter()
                .map(|profile| (profile.class.as_str(), profile.instances)),
        );
        let (added_predicates, removed_predicates, changed_predicates) = diff_counts(
            self.predicates
                .iter()
                .map(|profile| (profile.predicate.as_str(), profile.triples)),
            other
                .predicates
                .iter()
                .map(|profile| (profile.predicate.as_str(), profile.triples)),
        );

        SchemaDiff {
            triples_base: self.triples,
            triples_new: other.triples,
            added_classes,
            removed_classes,
            changed_classes,
            added_predicates,
            removed_predicates,
            changed_predicates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{diff_counts, DiffEntry};

    #[test]
    fn categories_are_sorted_by_iri_not_input_order() {
        let (added, removed, changed) = diff_counts(
            [
                ("urn:z-removed", 3),
                ("urn:b-changed", 2),
                ("urn:a-removed", 1),
            ],
            [("urn:y-added", 4), ("urn:b-changed", 5), ("urn:c-added", 6)],
        );

        assert_eq!(
            added,
            vec![
                DiffEntry {
                    iri: "urn:c-added".into(),
                    base: 0,
                    new: 6
                },
                DiffEntry {
                    iri: "urn:y-added".into(),
                    base: 0,
                    new: 4
                },
            ]
        );
        assert_eq!(
            removed,
            vec![
                DiffEntry {
                    iri: "urn:a-removed".into(),
                    base: 1,
                    new: 0
                },
                DiffEntry {
                    iri: "urn:z-removed".into(),
                    base: 3,
                    new: 0
                },
            ]
        );
        assert_eq!(
            changed,
            vec![DiffEntry {
                iri: "urn:b-changed".into(),
                base: 2,
                new: 5
            }]
        );
    }
}
