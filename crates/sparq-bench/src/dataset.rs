//! Deterministic synthetic dataset generator (a small WatDiv-flavoured social
//! graph): each entity is a `ex:Person` with a star of literal/IRI attributes
//! (name, age, city) plus several `ex:follows` edges, giving star joins, chains
//! and cycles (triangles) to exercise the different plan shapes.

/// Generates Turtle for `n` entities. The output is fully deterministic so the
/// two engines see byte-identical input and runs are reproducible.
pub fn generate(n: u32) -> String {
    let n = n.max(1);
    let follows_per = 4u32;
    let n_cities = (n / 10).max(1);

    // Pre-size: ~ (5 + follows_per) lines per entity, ~40 bytes each.
    let mut s = String::with_capacity(n as usize * (5 + follows_per as usize) * 40);
    s.push_str("@prefix ex: <http://ex/> .\n");

    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut rng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };

    for i in 0..n {
        s.push_str(&format!(
            "ex:n{i} a ex:Person ;\n  ex:name \"name{i}\" ;\n  ex:age {} ;\n  ex:city ex:c{} .\n",
            20 + i % 80,
            i % n_cities
        ));
        for _ in 0..follows_per {
            let t = rng() % n;
            s.push_str(&format!("ex:n{i} ex:follows ex:n{t} .\n"));
        }
    }
    s
}
