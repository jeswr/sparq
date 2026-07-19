//! T22 Prometheus metrics: a hand-rolled [text exposition format] endpoint —
//! no metrics crate, no extra dependencies.
//!
//! Counters are wired in at exactly two points: an outermost middleware
//! (`track`, applied *around* the hardening stack so shed/over-limit
//! responses are counted with their real status) records per-endpoint/status
//! request counts and the `/sparql` latency histogram; the update handler bumps
//! `sparq_updates_total`. The two gauges (graph triples, active subscriptions)
//! are read at scrape time — nothing is sampled in the background.
//!
//! [text exposition format]: https://prometheus.io/docs/instrumenting/exposition_formats/
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::http::AppState;

/// Upper bounds (seconds) of the fixed query-latency histogram buckets
/// (cumulative; `+Inf` is implicit via `_count`).
pub(crate) const QUERY_BUCKETS: [f64; 9] = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];

/// All server metrics. Cheap enough to update per request: the histogram and
/// counters are atomics; the per-(endpoint, status) map takes one short mutex
/// lock per request (bounded keys: a handful of endpoints × status codes).
#[derive(Default)]
pub struct Metrics {
    /// `sparq_http_requests_total{endpoint,status}`.
    requests: Mutex<BTreeMap<(&'static str, u16), u64>>,
    /// `sparq_query_duration_seconds` — cumulative bucket counts for `/sparql`.
    query_buckets: [AtomicU64; QUERY_BUCKETS.len()],
    query_count: AtomicU64,
    query_sum_nanos: AtomicU64,
    /// `sparq_updates_total` — successfully applied SPARQL updates.
    updates_total: AtomicU64,
}

impl Metrics {
    /// Records one finished HTTP request; `/sparql` requests also feed the
    /// latency histogram.
    pub(crate) fn record_request(&self, endpoint: &'static str, status: u16, elapsed: Duration) {
        *self
            .requests
            .lock()
            .expect("metrics lock poisoned")
            .entry((endpoint, status))
            .or_insert(0) += 1;
        if endpoint == "/sparql" {
            let secs = elapsed.as_secs_f64();
            for (i, le) in QUERY_BUCKETS.iter().enumerate() {
                if secs <= *le {
                    self.query_buckets[i].fetch_add(1, Ordering::Relaxed);
                }
            }
            self.query_count.fetch_add(1, Ordering::Relaxed);
            self.query_sum_nanos
                .fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);
        }
    }

    pub(crate) fn inc_updates(&self) {
        self.updates_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Renders the Prometheus text exposition; the two gauges are passed in by
    /// the scrape handler (read from live state at scrape time).
    pub(crate) fn render(&self, active_subscriptions: usize, graph_triples: usize) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("# HELP sparq_http_requests_total Total HTTP requests by endpoint and response status.\n");
        out.push_str("# TYPE sparq_http_requests_total counter\n");
        for ((endpoint, status), n) in self.requests.lock().expect("metrics lock poisoned").iter() {
            let _ = writeln!(
                out,
                "sparq_http_requests_total{{endpoint=\"{endpoint}\",status=\"{status}\"}} {n}"
            );
        }
        out.push_str("# HELP sparq_query_duration_seconds Wall time of /sparql requests (query + update operations).\n");
        out.push_str("# TYPE sparq_query_duration_seconds histogram\n");
        for (i, le) in QUERY_BUCKETS.iter().enumerate() {
            let _ = writeln!(
                out,
                "sparq_query_duration_seconds_bucket{{le=\"{le}\"}} {}",
                self.query_buckets[i].load(Ordering::Relaxed)
            );
        }
        let count = self.query_count.load(Ordering::Relaxed);
        let _ = writeln!(
            out,
            "sparq_query_duration_seconds_bucket{{le=\"+Inf\"}} {count}"
        );
        let _ = writeln!(
            out,
            "sparq_query_duration_seconds_sum {}",
            self.query_sum_nanos.load(Ordering::Relaxed) as f64 / 1e9
        );
        let _ = writeln!(out, "sparq_query_duration_seconds_count {count}");
        out.push_str(
            "# HELP sparq_active_subscriptions Currently active WebSocket subscriptions.\n",
        );
        out.push_str("# TYPE sparq_active_subscriptions gauge\n");
        let _ = writeln!(out, "sparq_active_subscriptions {active_subscriptions}");
        out.push_str("# HELP sparq_graph_triples Triples in the published graph.\n");
        out.push_str("# TYPE sparq_graph_triples gauge\n");
        let _ = writeln!(out, "sparq_graph_triples {graph_triples}");
        out.push_str("# HELP sparq_updates_total Successfully applied SPARQL updates.\n");
        out.push_str("# TYPE sparq_updates_total counter\n");
        let _ = writeln!(
            out,
            "sparq_updates_total {}",
            self.updates_total.load(Ordering::Relaxed)
        );
        out
    }
}

/// The fixed endpoint label a request path maps to (bounded cardinality: no raw
/// paths ever become label values).
fn endpoint_label(path: &str) -> &'static str {
    if path == "/sparql" {
        "/sparql"
    } else if path == "/sparql/graph" {
        "/sparql/graph"
    } else if path.starts_with("/graphs") {
        "/graphs"
    } else if path == "/subscriptions" {
        "/subscriptions"
    } else if path == "/health" {
        "/health"
    } else if path == "/metrics" {
        "/metrics"
    } else {
        "other"
    }
}

/// Outermost middleware: times every request and records it (endpoint label ×
/// final status) once the response — including hardening-layer rejections like
/// 429/413 — is known.
pub(crate) async fn track(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let endpoint = endpoint_label(req.uri().path());
    let start = std::time::Instant::now();
    let resp = next.run(req).await;
    state
        .metrics()
        .record_request(endpoint, resp.status().as_u16(), start.elapsed());
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposition_shape_and_histogram_cumulativity() {
        let m = Metrics::default();
        m.record_request("/sparql", 200, Duration::from_millis(2));
        m.record_request("/sparql", 200, Duration::from_millis(60));
        m.record_request("/health", 200, Duration::from_micros(10));
        m.record_request("/sparql", 400, Duration::from_micros(100));
        m.inc_updates();
        let text = m.render(3, 42);
        assert!(
            text.contains("sparq_http_requests_total{endpoint=\"/sparql\",status=\"200\"} 2"),
            "{text}"
        );
        assert!(
            text.contains("sparq_http_requests_total{endpoint=\"/sparql\",status=\"400\"} 1"),
            "{text}"
        );
        assert!(
            text.contains("sparq_http_requests_total{endpoint=\"/health\",status=\"200\"} 1"),
            "{text}"
        );
        // Buckets are cumulative: 0.001 holds the 100µs request; 0.005 adds 2ms;
        // 0.1 and up hold all three; +Inf == _count.
        assert!(
            text.contains("sparq_query_duration_seconds_bucket{le=\"0.001\"} 1"),
            "{text}"
        );
        assert!(
            text.contains("sparq_query_duration_seconds_bucket{le=\"0.005\"} 2"),
            "{text}"
        );
        assert!(
            text.contains("sparq_query_duration_seconds_bucket{le=\"0.1\"} 3"),
            "{text}"
        );
        assert!(
            text.contains("sparq_query_duration_seconds_bucket{le=\"+Inf\"} 3"),
            "{text}"
        );
        assert!(
            text.contains("sparq_query_duration_seconds_count 3"),
            "{text}"
        );
        assert!(text.contains("sparq_active_subscriptions 3"), "{text}");
        assert!(text.contains("sparq_graph_triples 42"), "{text}");
        assert!(text.contains("sparq_updates_total 1"), "{text}");
    }

    #[test]
    fn endpoint_labels_are_bounded() {
        assert_eq!(endpoint_label("/sparql"), "/sparql");
        assert_eq!(endpoint_label("/sparql/graph"), "/sparql/graph");
        assert_eq!(endpoint_label("/graphs/foo/bar"), "/graphs");
        assert_eq!(endpoint_label("/subscriptions"), "/subscriptions");
        assert_eq!(endpoint_label("/metrics"), "/metrics");
        assert_eq!(endpoint_label("/anything/else"), "other");
    }
}
