//! Embedding providers. The engine never runs a model in-process (design decision:
//! embeddings are produced out-of-process); this module defines the provider-agnostic
//! [`Embedder`] trait, a deterministic **test-only** [`HashEmbedder`], and — behind the
//! non-default `provider` feature — the API shape for a live OpenAI-compatible
//! `/v1/embeddings` provider whose HTTP transport is supplied by the caller.

/// A batch text-embedding provider. Implementations must be deterministic per input
/// within one store build (vectors written under different provider states do not
/// compare meaningfully).
pub trait Embedder {
    /// The dimension of every vector this embedder produces.
    fn dim(&self) -> usize;
    /// Embeds each text; returns one `dim()`-length vector per input, in input order.
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String>;
}

/// **Test-only** deterministic embedder: a random sign projection of character-trigram
/// and word hashes. Texts sharing n-grams get correlated vectors, so lexical similarity
/// becomes vector similarity — enough to exercise the store, the ANN indexes and the
/// label pipeline **without any model or network**. It captures NO semantics ("car" and
/// "automobile" are unrelated); do not ship it as a retrieval feature.
///
/// Per token `t`: a 64-bit FNV-1a hash seeds a SplitMix64 stream that contributes
/// `±1` to each of the `dim` components; the accumulated vector is L2-normalized.
/// Deterministic across platforms and runs for a fixed `(dim, seed)`.
pub struct HashEmbedder {
    dim: usize,
    seed: u64,
}

impl HashEmbedder {
    pub fn new(dim: usize) -> HashEmbedder {
        HashEmbedder::with_seed(dim, 0x5354_5156_5645_4354) // "SPQV"-flavored default
    }

    pub fn with_seed(dim: usize, seed: u64) -> HashEmbedder {
        assert!(dim > 0, "HashEmbedder dim must be > 0");
        HashEmbedder { dim, seed }
    }

    fn embed_one(&self, text: &str) -> Vec<f32> {
        let mut acc = vec![0f32; self.dim];
        let lower = text.to_lowercase();
        // Words…
        for word in lower.split_whitespace() {
            self.add_token(&mut acc, word.as_bytes());
        }
        // …and character trigrams over the whitespace-normalized, padded text, so
        // near-matches ("Athina" / "Athens 1896") still share mass.
        let padded: Vec<char> = std::iter::once(' ')
            .chain(lower.split_whitespace().collect::<Vec<_>>().join(" ").chars())
            .chain(std::iter::once(' '))
            .collect();
        for window in padded.windows(3) {
            let s: String = window.iter().collect();
            self.add_token(&mut acc, s.as_bytes());
        }
        let norm = acc.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut acc {
                *v /= norm;
            }
        }
        acc
    }

    fn add_token(&self, acc: &mut [f32], token: &[u8]) {
        let mut state = fnv1a64(token) ^ self.seed;
        for slot in acc.iter_mut() {
            *slot += if splitmix64(&mut state) & 1 == 1 { 1.0 } else { -1.0 };
        }
    }
}

impl Embedder for HashEmbedder {
    fn dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        Ok(texts.iter().map(|t| self.embed_one(t)).collect())
    }
}

#[inline]
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[inline]
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Live-provider API shape (non-default `provider` feature): an OpenAI-compatible
/// `/v1/embeddings` client where the **HTTP transport is supplied by the caller** —
/// this crate builds the request body and parses the response, but never opens a
/// socket (no HTTP client dependency; tests stay offline).
#[cfg(feature = "provider")]
pub mod provider {
    use super::Embedder;

    /// One HTTP POST with a JSON body, supplied by the caller over their HTTP client
    /// of choice (reqwest, ureq, a recorded cassette in tests, …). Returns the
    /// response body on 2xx, `Err` otherwise.
    pub trait Transport {
        fn post_json(
            &self,
            url: &str,
            headers: &[(String, String)],
            body: &str,
        ) -> Result<String, String>;
    }

    /// Configuration for an OpenAI-compatible `/v1/embeddings` endpoint.
    pub struct ProviderConfig {
        /// Full endpoint URL, e.g. `https://api.openai.com/v1/embeddings`.
        pub endpoint: String,
        /// Model name passed in the request body, e.g. `text-embedding-3-small`.
        pub model: String,
        /// Sent as `Authorization: Bearer …` when present.
        pub api_key: Option<String>,
        /// Expected vector dimension; responses with other dimensions are an error.
        pub dim: usize,
    }

    /// An [`Embedder`] over a remote OpenAI-compatible embeddings API.
    pub struct RemoteEmbedder<T: Transport> {
        cfg: ProviderConfig,
        transport: T,
    }

    impl<T: Transport> RemoteEmbedder<T> {
        pub fn new(cfg: ProviderConfig, transport: T) -> RemoteEmbedder<T> {
            RemoteEmbedder { cfg, transport }
        }
    }

    impl<T: Transport> Embedder for RemoteEmbedder<T> {
        fn dim(&self) -> usize {
            self.cfg.dim
        }

        fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
            if texts.is_empty() {
                return Ok(Vec::new());
            }
            let body = serde_json::json!({ "model": self.cfg.model, "input": texts });
            let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
            if let Some(key) = &self.cfg.api_key {
                headers.push(("Authorization".to_string(), format!("Bearer {key}")));
            }
            let response =
                self.transport.post_json(&self.cfg.endpoint, &headers, &body.to_string())?;
            let parsed: serde_json::Value =
                serde_json::from_str(&response).map_err(|e| format!("bad response JSON: {e}"))?;
            let data = parsed["data"]
                .as_array()
                .ok_or_else(|| "response has no `data` array".to_string())?;
            if data.len() != texts.len() {
                return Err(format!("asked for {} embeddings, got {}", texts.len(), data.len()));
            }
            // The API may return out of order; honor the `index` field. Each input
            // index must appear exactly once — a duplicate or missing index is a
            // protocol violation (the Embedder contract is one dim()-length vector
            // per input, in input order).
            let mut out: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
            for item in data {
                let i = item["index"].as_u64().ok_or("embedding without `index`")? as usize;
                let emb = item["embedding"].as_array().ok_or("embedding without `embedding`")?;
                let v: Vec<f32> =
                    emb.iter().map(|x| x.as_f64().unwrap_or(f64::NAN) as f32).collect();
                if v.len() != self.cfg.dim || v.iter().any(|x| !x.is_finite()) {
                    return Err(format!(
                        "embedding {i} has dim {} (expected {}) or non-finite values",
                        v.len(),
                        self.cfg.dim
                    ));
                }
                let slot = out.get_mut(i).ok_or(format!("embedding index {i} out of range"))?;
                if slot.is_some() {
                    return Err(format!("duplicate embedding index {i} in response"));
                }
                *slot = Some(v);
            }
            out.into_iter()
                .enumerate()
                .map(|(i, v)| v.ok_or(format!("response is missing embedding index {i}")))
                .collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct Fake;
        impl Transport for Fake {
            fn post_json(&self, url: &str, headers: &[(String, String)], body: &str) -> Result<String, String> {
                assert_eq!(url, "https://example.test/v1/embeddings");
                assert!(headers.iter().any(|(k, v)| k == "Authorization" && v == "Bearer k"));
                assert!(body.contains("\"model\":\"m\""));
                // Out-of-order response to exercise `index` handling.
                Ok(r#"{"data":[
                    {"index":1,"embedding":[0.5,0.5]},
                    {"index":0,"embedding":[1.0,0.0]}
                ]}"#
                .to_string())
            }
        }

        #[test]
        fn remote_embedder_shapes_request_and_orders_response() {
            let e = RemoteEmbedder::new(
                ProviderConfig {
                    endpoint: "https://example.test/v1/embeddings".into(),
                    model: "m".into(),
                    api_key: Some("k".into()),
                    dim: 2,
                },
                Fake,
            );
            let got = e.embed(&["a", "b"]).unwrap();
            assert_eq!(got, vec![vec![1.0, 0.0], vec![0.5, 0.5]]);
        }

        /// A duplicate `index` in the response (which also leaves another index
        /// missing) must be rejected, not silently yield an empty vector.
        struct DupIndex;
        impl Transport for DupIndex {
            fn post_json(&self, _: &str, _: &[(String, String)], _: &str) -> Result<String, String> {
                Ok(r#"{"data":[
                    {"index":1,"embedding":[0.5,0.5]},
                    {"index":1,"embedding":[1.0,0.0]}
                ]}"#
                .to_string())
            }
        }

        #[test]
        fn remote_embedder_rejects_duplicate_response_indexes() {
            let e = RemoteEmbedder::new(
                ProviderConfig {
                    endpoint: "https://example.test/v1/embeddings".into(),
                    model: "m".into(),
                    api_key: None,
                    dim: 2,
                },
                DupIndex,
            );
            let err = e.embed(&["a", "b"]).unwrap_err();
            assert!(err.contains("duplicate embedding index 1"), "got: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_embedder_is_deterministic_and_normalized() {
        let e = HashEmbedder::new(32);
        let a = e.embed(&["Usain Bolt"]).unwrap();
        let b = e.embed(&["Usain Bolt"]).unwrap();
        assert_eq!(a, b);
        let norm: f32 = a[0].iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "norm = {norm}");
    }

    #[test]
    fn hash_embedder_correlates_lexically_similar_texts() {
        let e = HashEmbedder::new(64);
        let v = e.embed(&["Usain Bolt", "Usain Bolt Jr", "Pierre de Coubertin"]).unwrap();
        let cos = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
        let near = cos(&v[0], &v[1]);
        let far = cos(&v[0], &v[2]);
        assert!(near > far, "near {near} should beat far {far}");
        assert!(near > 0.4, "shared n-grams should correlate strongly, got {near}");
    }

    #[test]
    fn hash_embedder_empty_text_is_zero_vector() {
        let e = HashEmbedder::new(16);
        let v = e.embed(&[""]).unwrap();
        assert!(v[0].iter().all(|&x| x == 0.0));
    }
}
