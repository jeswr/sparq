//! Test chunk 0 for fn:id
//! Contains 25 tests

use dep::xpath::stub_fnid;

#[test]
fn fn_id_dtd_5() {
    // Evaluation of fn:id with given IDREF matching a single element.
    // XPath: fn:id(\"id1\", /IDS[1])/string(@anId)
    // Expected: id1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_6() {
    // Evaluation of fn:id with given IDREF not matching a single element.
    // XPath: fn:count(fn:id(\"nomatchingid\", /IDS[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_7() {
    // Evaluation of fn:id with given IDREF matching same element.
    // XPath: fn:id(\"id2 id2\", /IDS[1])/name()
    // Expected: elementwithid-2
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_8() {
    // Evaluation of fn:id with multiple IDREF matching multiple elements.
    // XPath: fn:id(\"id1 id2\", /IDS[1])/name()
    // Expected: elementwithid-1 elementwithid-2
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_9() {
    // Evaluation of fn:id with multiple IDREF, but only one matching one element.
    // XPath: fn:id(\"id1 nomatching\", /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_10() {
    // Evaluation of fn:id with multiple IDREF, but none matching one element.
    // XPath: fn:count(fn:id(\"nomatching1 nomatching2\", /IDS[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_12() {
    // Evaluation of fn:id function that give first argument as argument to fn:substrin
    // XPath: fn:id(fn:substring(\"1id3\",2), /IDS[1])/name()
    // Expected: elementwithid-3
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_13() {
    // Evaluation of fn:id, where the same IDREF makes reference to the same element.
    // XPath: fn:id(\"id4\", /IDS[1])/name()
    // Expected: elementwithid-4
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_15() {
    // Evaluation of fn:id for which IDREF list have the same value for all its members
    // XPath: fn:id(\"id1 id1\", /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_16() {
    // Evaluation of fn:id for which IDREF list have the same value for all its members
    // XPath: fn:id(\"id1 ID1\", /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_17() {
    // Evaluation of fn:id for which IDREF uses the fn lower case function.
    // XPath: fn:id(fn:lower-case(\"ID1\"), /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_18() {
    // Evaluation of fn:id for which IDREF uses the fn:upper-case function.
    // XPath: fn:id(fn:upper-case(\"id5\"), /IDS[1])/name()
    // Expected: elementwithid-6
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_19() {
    // Evaluation of fn:id for which $arg uses the fn:concat function.
    // XPath: fn:id(fn:concat(\"i\",\"d1\"), /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_20() {
    // Evaluation of fn:id for which $arg uses the xs:string function.
    // XPath: fn:id(xs:string(\"id1\"), /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_21() {
    // Evaluation of fn:id for which $arg uses the fn:string-join function.
    // XPath: fn:id(fn:string-join((\"id\",\"1\"),\"\"), /IDS[1])/name()
    // Expected: elementwithid-1
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn fn_id_dtd_23() {
    // Evaluation of fn:id together with declare ordering.
    // XPath: declare ordering ordered; fn:id(\"id1 id2\", /IDS[1])/name()
    // Expected: elementwithid-1 elementwithid-2
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_9() {
    // Use a constructed document node with xml:id.
    // XPath: let $i := document {<e> <e/> <e/> <e/> <e/> <e/> <e/> <e/> <
    // Expected: b
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_10() {
    // Select elements with xml:id, and trigger node sorting.
    // XPath: for $i in id((\"short\", \"positiveInteger\")) return $i/@na
    // Expected: positiveInteger short
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_11() {
    // Select elements with xml:id, and trigger node sorting.
    // XPath: id((\"short\"), //xs:element/@name[. = \"positiveInteger\"])
    // Expected: short
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_12() {
    // Select elements with xml:id, trigger node sorting, and use an invalid NCName in
    // XPath: id((\".\", \"short\", \"123\"), //xs:element/@name[. = \"pos
    // Expected: short
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_13() {
    // Use on xml:id with input strings which contains many IDREFs, and invalid ones to
    // XPath: fn:id(//b/@ref)/data(exactly-one(@*))
    // Expected: a b c d e f i
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_14() {
    // Select elements with xml:id, and trigger node sorting: space-separated id list
    // XPath: for $i in id((\"short positiveInteger\")) return $i/@name/st
    // Expected: positiveInteger short
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn k2_seqidfunc_15() {
    // Use on xml:id with input strings which contains many IDREFs, and invalid ones to
    // XPath: fn:id(string-join(reverse(//b/@ref), ' '))/data(exactly-one(
    // Expected: a b c d e f i
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn cbcl_id_001() {
    //  test fn:id on the empty sequence
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}

#[test]
fn cbcl_id_003() {
    //  test fn:id in an axis expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:id is implemented
    let _ = stub_fnid();
}
