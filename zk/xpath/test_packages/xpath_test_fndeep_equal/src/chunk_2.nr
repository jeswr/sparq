//! Test chunk 2 for fn:deep-equal
//! Contains 50 tests

use dep::xpath::stub_fndeep_equal;

#[test]
fn fn_deep_equal_arrays_3() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([[]], [[]])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_4() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([], [1])
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_5() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([1], 1)
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_6() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([], [()])
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_7() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([()], [()])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_8() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([map{}], [map{}])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_9() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([map{1:1, 2:2}], [map{2:2, 1:1}])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_10() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([<a>1</a>], [<a>1</a>])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_11() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([['a', 'b', 'c']], [['a', 'b', 'c']])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_12() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([['a', 'b', 'c']], [['A', 'B', 'C']])
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_13() {
    //  Apply deep-equal to arrays, with a collation
    // XPath: fn:deep-equal([['a', 'b', 'c']], [['A', 'B', 'C']], 'http://
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_14() {
    //  Apply deep-equal to a sequence of arrays
    // XPath: fn:deep-equal(([['a', 'b', 'c']], [], [1]), ([['a', 'b', 'c'
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_15() {
    //  Apply deep-equal to a sequence of arrays
    // XPath: fn:deep-equal(([['a', 'b', 'c']], [], [1]), ([['a', 'b', 'c'
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_16() {
    //  Apply deep-equal to a sequence of arrays
    // XPath: fn:deep-equal(([['a', 'b', 'c']], [], [1]), ([['a', 'b', 'c'
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_17() {
    //  Apply deep-equal to a sequence of arrays
    // XPath: fn:deep-equal(([['a', 'b', 'c']], [], [1]), ([['a', 'b', 'c'
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_18() {
    //  Apply deep-equal to a sequence of arrays, after remove operation
    // XPath: fn:deep-equal(([['a', 'b', 'd'] => array:put(3, 'c')], [], [
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_1() {
    //  Map deep-equal to map
    // XPath: fn:deep-equal(map{}, map{})
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_2() {
    //  Map deep-equal to map
    // XPath: fn:deep-equal(map{1:2}, map{})
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_3() {
    //  Map deep-equal to map
    // XPath: fn:deep-equal(map{}, map{1:()})
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_4() {
    //  Map deep-equal to map; order of entries is ignored
    // XPath: fn:deep-equal(map{1:true(), 2:false()}, map{2:false(), 1:tru
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_5() {
    //  Map deep-equal to map; keys differ in type but equal in value
    // XPath: fn:deep-equal(map{1:true()}, map{1.0:true()})
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_6() {
    //  Map deep-equal to map; keys differ in type but equal in value
    // XPath: fn:deep-equal(map{1:true()}, map{1.0e0:true()})
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_7() {
    //  Map deep-equal to map; NaN keys
    // XPath: fn:deep-equal(map{xs:double('NaN'):true()}, map{xs:double('N
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_8() {
    //  Map deep-equal to map; NaN keys
    // XPath: fn:deep-equal(map{xs:double('NaN'):true()}, map{xs:float('Na
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_9() {
    //  Map deep-equal to map; value is an array
    // XPath: fn:deep-equal(map{12:[1,2,3]}, map{12.0:[1.0, 2.0, 3.0]})
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_10() {
    //  Map deep-equal to map; values are unequal arrays
    // XPath: fn:deep-equal(map{12:[1,2,3]}, map{12:[3,2,1]})
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_11() {
    //  Map deep-equal to map; values are nodes
    // XPath: fn:deep-equal(map{12:<a x='2' y='3'>23</a>}, map{12:<a y='3'
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_12() {
    //  Map deep-equal to map; values are nodes
    // XPath: fn:deep-equal(map{12:<a x='2' y='3'>23</a>}, map{12:<a y='3'
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_13() {
    //  Map deep-equal to map; collation is not used for comparing keys
    // XPath: fn:deep-equal(map{\"a\":1}, map{\"A\":1},              'http
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_14() {
    //  Map deep-equal to map; collation is used for comparing values
    // XPath: fn:deep-equal(map{1:\"a\"}, map{1:\"A\"},              'http
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_15() {
    //  Map deep-equal to map; after remove operation
    // XPath: fn:deep-equal(map{1:\"A\"}, map:remove(map{1:\"A\", 2:\"B\"}
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_16() {
    //  Map deep-equal to map; effect of implicit timezone
    // XPath: let $t := xs:dateTime('2014-10-14T11:31:12') return
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_17() {
    //  Map deep-equal to map; effect of implicit timezone
    // XPath: let $t := xs:dateTime('2014-10-14T11:31:12') return
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_18() {
    //  Sequence of maps deep-equal to a sequence of maps
    // XPath: fn:deep-equal((map{1:'a', 2:'b'}, map{1:'x', 2:'y'}), (map{2
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_19() {
    //  Sequence of maps deep-equal to a sequence of maps
    // XPath: fn:deep-equal((map{1:'a', 2:'b'}, map{1:'x', 2:'y'}), (map{2
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_maps_20() {
    //  Sequence of maps deep-equal to a sequence of maps
    // XPath: fn:deep-equal((map{1:'a', 2:'b'}, map{1:'x', 2:'y'}), (map{2
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_node_args_3() {
    //  Apply deep-equal to nodes
    // XPath: fn:deep-equal(<a> {/node(), /node()} </a>/node(), <b> {/node
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_node_args_4() {
    //  Apply deep-equal to nodes
    // XPath: fn:deep-equal(<a> {/node(), <diff x='1'/>, /node()} </a>/nod
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_node_args_5() {
    //  Apply deep-equal to nodes
    // XPath: fn:deep-equal(<a> {/node(), <diff x='1'/>, /node()} </a>/nod
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_6() {
    //  A test whose essence is: `deep-equal('a string', 'a string', 'http://www.w3.org
    // XPath: deep-equal(\"a string\", \"a string\", \"http://www.w3.org/2
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_7() {
    //  A test whose essence is: `deep-equal((), ()) eq true()`.
    // XPath: deep-equal((), ()) eq true()
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_8() {
    //  A test whose essence is: `deep-equal(xs:float('NaN'), xs:float('NaN'))`.
    // XPath: deep-equal(xs:float(\"NaN\"), xs:float(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_9() {
    //  A test whose essence is: `deep-equal(xs:double('NaN'), xs:double('NaN'))`.
    // XPath: deep-equal(xs:double(\"NaN\"), xs:double(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_10() {
    //  A test whose essence is: `deep-equal(xs:float('NaN'), xs:double('NaN'))`.
    // XPath: deep-equal(xs:float(\"NaN\"), xs:double(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_11() {
    //  A test whose essence is: `deep-equal(xs:double('NaN'), xs:float('NaN'))`.
    // XPath: deep-equal(xs:double(\"NaN\"), xs:float(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_12() {
    //  A test whose essence is: `not(deep-equal(xs:float('NaN'), xs:float(0)))`.
    // XPath: not(deep-equal(xs:float(\"NaN\"), xs:float(0)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_13() {
    //  A test whose essence is: `not(deep-equal(xs:float(0), xs:float('NaN')))`.
    // XPath: not(deep-equal(xs:float(0), xs:float(\"NaN\")))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_14() {
    //  A test whose essence is: `not(deep-equal(xs:double(0), xs:double('NaN')))`.
    // XPath: not(deep-equal(xs:double(0), xs:double(\"NaN\")))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_15() {
    //  A test whose essence is: `not(deep-equal(xs:double('NaN'), xs:double(0)))`.
    // XPath: not(deep-equal(xs:double(\"NaN\"), xs:double(0)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_16() {
    //  A test whose essence is: `not(deep-equal(xs:decimal('1'), xs:anyURI('example.co
    // XPath: not(deep-equal(xs:decimal(\"1\"), xs:anyURI(\"example.com\")
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}
