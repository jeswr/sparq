#![cfg(feature = "profile")]
use sparq_core::dict::{Dict, Id};
use sparq_reason::{materialize, materialize_profiled, Profile};
use std::collections::BTreeSet;

fn fixture() -> (Dict, Vec<[Id; 3]>) {
    let mut d = Dict::new();
    let ty = d.intern_iri(oxrdf::vocab::rdf::TYPE.as_str());
    let transitive = d.intern_iri("http://www.w3.org/2002/07/owl#TransitiveProperty");
    let p = d.intern_iri("http://example.com/path");
    let ns: Vec<_> = (0..12)
        .map(|i| d.intern_iri(&format!("http://example.com/n{i}")))
        .collect();
    let mut ts = vec![[p, ty, transitive]];
    ts.extend(ns.windows(2).map(|w| [w[0], p, w[1]]));
    (d, ts)
}

#[test]
fn explosive_rule_is_top_without_changing_closure() {
    let (mut da, mut a) = fixture();
    let (mut db, mut b) = fixture();
    let plain = materialize(Profile::OwlRl, &mut da, &mut a);
    let (profiled, report) = materialize_profiled(Profile::OwlRl, &mut db, &mut b);
    assert_eq!(plain, profiled);
    assert_eq!(
        a.into_iter().collect::<BTreeSet<_>>(),
        b.into_iter().collect()
    );
    let top = report.top(1);
    assert_eq!(top[0].rule, "OWL RL fixpoint rules");
    assert!(top[0].derived_count > 40);
}
