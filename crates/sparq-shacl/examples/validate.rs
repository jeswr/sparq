//! CLI-style SHACL validation:
//!
//! ```sh
//! cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl [--turtle]
//! ```
//!
//! Prints the validation report (text by default, `--turtle` for the report
//! graph) and exits 0 iff the data conforms.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (files, turtle): (Vec<&String>, bool) = (
        args.iter().filter(|a| !a.starts_with("--")).collect(),
        args.iter().any(|a| a == "--turtle"),
    );
    let [data_path, shapes_path] = files[..] else {
        eprintln!("usage: validate <data.ttl> <shapes.ttl> [--turtle]");
        std::process::exit(2);
    };
    let load = |path: &str| -> sparq_core::Graph {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        // Resolve any relative IRIs against the file's own location.
        sparq_shacl::load_turtle_with_base(&text, &format!("file://{path}"))
            .unwrap_or_else(|e| panic!("parse {path}: {e}"))
    };
    let report = sparq_shacl::validate(&load(data_path), &load(shapes_path));
    if turtle {
        print!("{}", report.to_turtle());
    } else {
        print!("{}", report.to_text());
    }
    std::process::exit(i32::from(!report.conforms));
}
