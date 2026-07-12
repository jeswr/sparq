#![cfg(feature = "window-aggregate")]

// [GPT-5.6] sq-lsp7k.27: hand-derived deterministic aggregate oracle.

use oxrdf::{Literal, NamedNode, Term, Variable};
use sparq_rsp::{window_aggregate, Agg, WindowResult};

fn result(vars: &[&str], rows: Vec<Vec<Option<Term>>>) -> WindowResult {
    WindowResult {
        start: 0,
        end: 10,
        vars: vars
            .iter()
            .map(|name| Variable::new(*name).expect("test variable is valid"))
            .collect(),
        rows,
    }
}

fn number(value: i32) -> Option<Term> {
    Some(Literal::from(value).into())
}

#[test]
fn exact_count_sum_min_and_max_oracle() {
    let window = result(
        &["x"],
        vec![vec![number(1)], vec![number(2)], vec![number(3)]],
    );

    assert_eq!(window_aggregate(&window, "x", Agg::Count), Some(3.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Sum), Some(6.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Min), Some(1.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Max), Some(3.0));
}

#[test]
fn numeric_fold_skips_unbound_and_non_numeric_while_count_keeps_rows() {
    let window = result(
        &["x"],
        vec![
            vec![number(4)],
            vec![None],
            vec![Some(Literal::new_simple_literal("not numeric").into())],
            vec![Some(
                NamedNode::new_unchecked("http://example/value").into(),
            )],
            Vec::new(),
            vec![number(-2)],
        ],
    );

    assert_eq!(window_aggregate(&window, "x", Agg::Count), Some(6.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Sum), Some(2.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Min), Some(-2.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Max), Some(4.0));
}

#[test]
fn absent_variable_returns_none_for_every_aggregate() {
    let window = result(&["x"], vec![vec![number(1)]]);

    for aggregate in [Agg::Count, Agg::Sum, Agg::Min, Agg::Max] {
        assert_eq!(window_aggregate(&window, "notavar", aggregate), None);
    }
}

#[test]
fn empty_window_has_zero_count_and_sum_but_no_extrema() {
    let window = result(&["x"], Vec::new());

    assert_eq!(window_aggregate(&window, "x", Agg::Count), Some(0.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Sum), Some(0.0));
    assert_eq!(window_aggregate(&window, "x", Agg::Min), None);
    assert_eq!(window_aggregate(&window, "x", Agg::Max), None);
}

#[test]
fn column_selection_and_row_order_do_not_change_the_oracle() {
    let first = result(
        &["ignored", "x"],
        vec![
            vec![number(100), Some(Literal::from(1.0e16).into())],
            vec![number(100), Some(Literal::from(-1.0e16).into())],
            vec![number(100), number(1)],
        ],
    );
    let reversed = result(
        &["ignored", "x"],
        vec![
            vec![number(100), number(1)],
            vec![number(100), Some(Literal::from(-1.0e16).into())],
            vec![number(100), Some(Literal::from(1.0e16).into())],
        ],
    );

    for aggregate in [Agg::Count, Agg::Sum, Agg::Min, Agg::Max] {
        assert_eq!(
            window_aggregate(&first, "x", aggregate),
            window_aggregate(&reversed, "x", aggregate)
        );
    }
}
