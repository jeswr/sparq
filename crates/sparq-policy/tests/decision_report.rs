#![cfg(feature = "decision-report")]

use sparq_policy::{report::DecisionReport, Decision};

// [GPT-5.6] sq-mu4au: every input exercises a distinct aggregation branch. Removing
// any counter update, conflict recognition, or BTreeMap ordering makes this test fail.
#[test]
fn fixed_decision_batch_has_stable_complete_report() {
    let permitted = Decision {
        allow: true,
        matched_rules: vec!["permission-read".into()],
        unmet_constraints: Vec::new(),
    };
    let denied = Decision {
        allow: false,
        matched_rules: Vec::new(),
        unmet_constraints: vec!["no permission matches the request".into()],
    };
    let conflict = Decision {
        allow: false,
        matched_rules: vec!["prohibition-write".into()],
        unmet_constraints: vec!["prohibition prohibition-write matches the request".into()],
    };
    let escaped_action = "urn:action/a\"\\\n";
    let inputs = [
        ("urn:action/read", &permitted),
        ("urn:action/read", &denied),
        (escaped_action, &conflict),
    ];

    let report = DecisionReport::summarize(inputs);
    assert_eq!(report.total, 3);
    assert_eq!(report.permitted, 1);
    assert_eq!(report.denied, 2);
    assert_eq!(report.conflicts, 1);
    assert_eq!(report.permitted + report.denied, report.total);
    assert_eq!(report.per_action.len(), 2);
    assert_eq!(
        report
            .per_action
            .iter()
            .map(|row| row.permitted)
            .sum::<usize>(),
        1
    );
    assert_eq!(
        report
            .per_action
            .iter()
            .map(|row| row.denied)
            .sum::<usize>(),
        2
    );
    assert_eq!(report.per_action[0].action, escaped_action);

    let expected = concat!(
        "{\"total\":3,\"permitted\":1,\"denied\":2,\"conflicts\":1,",
        "\"per_action\":[{\"action\":\"urn:action/a\\\"\\\\\\n\",\"permitted\":0,\"denied\":1},",
        "{\"action\":\"urn:action/read\",\"permitted\":1,\"denied\":1}]}"
    );
    assert_eq!(report.to_json(), expected);
    assert_eq!(report.to_json().as_bytes(), report.to_json().as_bytes());
}
