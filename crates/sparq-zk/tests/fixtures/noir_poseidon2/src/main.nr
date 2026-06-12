// Cross-test vector generator: prints Poseidon2 outputs that
// crates/sparq-zk/tests/poseidon2_noir_cross.rs asserts the Rust
// implementation reproduces byte-for-byte. Run via `nargo execute`.
use poseidon::poseidon2::Poseidon2;

fn main() {
    // Permutation (the blackbox the Rust port mirrors).
    let p: [Field; 4] = std::hash::poseidon2_permutation([11, 22, 33, 44]);
    println(p);

    // Variable-length sponge hashes (the commitment pipeline's h_2/h_3 and
    // the flat per-graph commitment fold).
    println(Poseidon2::hash([1], 1));
    println(Poseidon2::hash([1, 2], 2));
    println(Poseidon2::hash([1, 2, 3], 3));
    println(Poseidon2::hash([1, 2, 3, 4], 4));
    println(Poseidon2::hash([5, 6, 7, 8, 9, 10, 11], 7));
    // A 40-leaf commitment-sized fold (credential scale).
    let mut leaves = [0; 40];
    for i in 0..40 {
        leaves[i] = (i as Field) + 1;
    }
    println(Poseidon2::hash(leaves, 40));
}
