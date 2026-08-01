// [GPT-5] #3557 — evaluate the site's exact Turtle/request pair through the real
// sparq-policy implementation and emit the Decision as machine-comparable JSON.
use sparq_policy::{evaluate, parse_policy_str, Request, Value};
use std::{env, fs};

fn main() {
    let mut args = env::args().skip(1);
    let turtle_path = args.next().expect("Turtle path");
    let action = args.next().expect("action");
    let target = args.next().expect("target");
    let party = args.next().expect("party");

    let turtle = fs::read_to_string(turtle_path).expect("read Turtle fixture");
    let policy = parse_policy_str(&turtle, "turtle").expect("parse site policy");
    let mut request = Request::new(action);
    if target != "-" {
        request = request.on(target);
    }
    if party != "-" {
        request = request.by(party);
    }

    while let Some(kind) = args.next() {
        match kind.as_str() {
            "context" => {
                let value_kind = args.next().expect("context value kind");
                let left = args.next().expect("context left operand");
                let value = args.next().expect("context value");
                let value = match value_kind.as_str() {
                    "datetime" => Value::DateTime(value),
                    "iri" => Value::Iri(value),
                    "number" => Value::Num(value.parse().expect("numeric context value")),
                    "string" => Value::Str(value),
                    other => panic!("unknown context value kind: {other}"),
                };
                request = request.with(left, value);
            }
            "discharge" => request = request.discharge(args.next().expect("duty IRI")),
            other => panic!("unknown request argument: {other}"),
        }
    }

    let decision = evaluate(&policy, &request);
    println!(
        "{}",
        serde_json::json!({
            "allow": decision.allow,
            "matched": decision.matched_rules,
            "unmet": decision.unmet_constraints,
        })
    );
}
