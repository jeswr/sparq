//! [FABLE-5] (sq-tonhr.2, epic sq-tonhr) PLUGGABLE CANDIDATE parser rows for the
//! `bench-nt` / `bench-ttl` lanes: the slot an rdf-shuttle GENERATED parser
//! (sq-tonhr.8/.9) drops into to be measured as a competitor row against the
//! incumbents (native `nt.rs`, oxttl) on the same corpus, in the same table and the
//! same `--json` emit — no harness surgery at measurement time.
//!
//! A candidate is registered here (a `Candidate` entry in [`candidates_for`]) behind a
//! cargo feature, so the DEFAULT build has ZERO candidate rows and byte-for-byte
//! unchanged output (the strictly-additive posture the `--json` emit already follows).
//! The `candidate-demo` feature compiles a reference-shaped demo candidate (a plain
//! serial oxttl wrapper) that exists to (a) prove the slot end-to-end and (b) document
//! the exact contract a real generated parser implements.
//!
//! CONTRACT: `parse` consumes the whole document text and returns the number of
//! statements parsed, erroring on invalid input. The harness VERIFIES the returned
//! count against the reference count before timing — a candidate that mis-parses the
//! corpus panics loudly instead of posting a bogus throughput row. (Quad-set-level
//! correctness is gated separately and much more strongly by
//! `sparq-conformance`'s `tests/parser_differential.rs`; a bench row is not a
//! correctness certificate.)

/// One pluggable candidate parser row.
pub struct Candidate {
    /// Row label, e.g. `shuttle-nt-v1`.
    pub name: &'static str,
    /// Whole-document parse returning the statement count (see module contract).
    pub parse: fn(&str) -> Result<usize, String>,
}

/// The candidate rows registered for `format` (`"ntriples"` | `"turtle"`; the future
/// NQ/TriG lanes will query the same registry). Empty in the default build.
pub fn candidates_for(format: &str) -> Vec<Candidate> {
    #[cfg(feature = "candidate-demo")]
    {
        match format {
            "ntriples" => vec![Candidate { name: "demo-oxttl-nt", parse: demo_oxttl_nt }],
            "turtle" => vec![Candidate { name: "demo-oxttl-ttl", parse: demo_oxttl_ttl }],
            _ => Vec::new(),
        }
    }
    #[cfg(not(feature = "candidate-demo"))]
    {
        let _ = format;
        Vec::new()
    }
}

/// Demo candidate: serial oxttl N-Triples (the slot template — a real generated
/// parser replaces the body with its own entry point).
#[cfg(feature = "candidate-demo")]
fn demo_oxttl_nt(text: &str) -> Result<usize, String> {
    let mut n = 0usize;
    for t in oxttl::NTriplesParser::new().for_slice(text.as_bytes()) {
        t.map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}

/// Demo candidate: serial oxttl Turtle.
#[cfg(feature = "candidate-demo")]
fn demo_oxttl_ttl(text: &str) -> Result<usize, String> {
    let mut n = 0usize;
    for t in oxttl::TurtleParser::new().for_slice(text.as_bytes()) {
        t.map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default build: the registry is EMPTY for every format — no candidate row, so
    /// the bench output stays byte-for-byte unchanged.
    #[cfg(not(feature = "candidate-demo"))]
    #[test]
    fn default_build_has_no_candidate_rows() {
        for f in ["ntriples", "turtle", "nquads", "trig"] {
            assert!(candidates_for(f).is_empty(), "unexpected candidate row for {f}");
        }
    }

    /// `candidate-demo` build: the demo candidates are registered and honour the
    /// contract (statement count on valid input, `Err` on invalid input).
    #[cfg(feature = "candidate-demo")]
    #[test]
    fn demo_candidates_honour_the_contract() {
        let nt = candidates_for("ntriples");
        assert_eq!(nt.len(), 1);
        assert_eq!(nt[0].name, "demo-oxttl-nt");
        let doc = "<http://ex/s> <http://ex/p> <http://ex/o> .\n\
                   <http://ex/s> <http://ex/p> \"v\" .\n";
        assert_eq!((nt[0].parse)(doc), Ok(2));
        assert!((nt[0].parse)("not rdf\n").is_err());

        let ttl = candidates_for("turtle");
        assert_eq!(ttl.len(), 1);
        assert_eq!((ttl[0].parse)("@prefix : <http://ex/> .\n:s :p :o .\n"), Ok(1));
        // Unwired formats have no demo row yet.
        assert!(candidates_for("nquads").is_empty());
    }
}
