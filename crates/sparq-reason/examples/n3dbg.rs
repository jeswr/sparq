fn main() {
    let path = std::env::args().nth(1).expect("path");
    let base = std::env::args().nth(2);
    let src = std::fs::read_to_string(&path).unwrap();
    let c = sparq_reason::n3::reason_n3_terms_with_resolver(&src, base.as_deref(), None).unwrap();
    let mut rows: Vec<String> = c.facts.iter().map(|r| format!("{r:?}")).collect();
    rows.sort();
    for r in rows {
        println!("{r}");
    }
}
