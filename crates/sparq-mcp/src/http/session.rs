//! The MCP Streamable HTTP **session registry**: one entry per `Mcp-Session-Id`, each
//! owning the queue of server→client messages its SSE stream drains and a bounded ring
//! of already-delivered events for `Last-Event-ID` resumption.
//!
//! [SONNET-4.6] (sq-2c0f0) Sessions are what make the transport *multiplexed*: many
//! concurrent clients each hold their own session and their own outbound stream, while
//! all of them are served from the one shared [`crate::McpServer`] (one dataset, one
//! lock). A session carries **no per-client authorization** — it is a correlation and
//! delivery handle, not a principal. The crate has no authentication at all, and adding
//! a session id does not change that; see the README trust model.

use std::collections::{HashMap, VecDeque};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// Why an SSE stream could not be opened.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StreamError {
    /// No such session (never created, or already terminated).
    NoSession,
    /// This session already has an open SSE stream. The MCP Streamable HTTP transport
    /// allows at most one GET stream per session, so a second is refused rather than
    /// silently splitting the message order between two sockets.
    AlreadyOpen,
}

/// The outcome of waiting for outbound messages on one session's stream.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Poll {
    /// One or more messages, each with the SSE `id` the client may resume from.
    Events(Vec<(u64, String)>),
    /// Nothing arrived before the keepalive deadline — send an SSE comment.
    Idle,
    /// The session is gone (terminated by `DELETE`, or the registry shut down); the
    /// stream must end.
    Gone,
}

#[derive(Debug, Default)]
struct Session {
    /// The id the *next* queued message will carry. Monotonic per session, which is
    /// what makes `Last-Event-ID` resumption well-defined.
    next_event_id: u64,
    /// Queued but not yet written to a stream.
    pending: VecDeque<(u64, String)>,
    /// Already written, kept for replay. Bounded by `replay_capacity`.
    delivered: VecDeque<(u64, String)>,
    /// Whether a GET stream currently holds this session.
    stream_open: bool,
}

/// The shared registry. Every method takes `&self`: it is meant to be held in an
/// `Arc` and used from the accept loop, from each connection thread, and from an
/// embedder thread pushing notifications.
#[derive(Debug)]
pub(crate) struct SessionRegistry {
    inner: Mutex<HashMap<String, Session>>,
    /// Signalled whenever a message is queued or a session disappears, so a stream
    /// blocked in [`SessionRegistry::poll`] wakes promptly instead of polling.
    signal: Condvar,
    replay_capacity: usize,
}

impl SessionRegistry {
    /// A registry whose per-session replay ring holds at most `replay_capacity`
    /// delivered events (`0` disables resumption).
    pub(crate) fn new(replay_capacity: usize) -> Self {
        SessionRegistry {
            inner: Mutex::new(HashMap::new()),
            signal: Condvar::new(),
            replay_capacity,
        }
    }

    /// A poisoned registry lock means a thread panicked while holding it. The map is
    /// still structurally sound (every mutation below is a whole-map operation), so
    /// recovering is preferable to poisoning every subsequent request.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Session>> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Register `id`, refusing (and changing nothing) if it already exists or if the
    /// registry already holds `max` sessions. `max == 0` means no cap.
    ///
    /// The capacity check is inside the lock on purpose: nothing authenticates a
    /// client, so "open a connection, POST `initialize`, repeat" is available to anyone
    /// who can reach the listener, and a check outside the lock would let concurrent
    /// handshakes race past the cap.
    pub(crate) fn create(&self, id: &str, max: usize) -> bool {
        let mut sessions = self.lock();
        if sessions.contains_key(id) {
            return false;
        }
        if max > 0 && sessions.len() >= max {
            return false;
        }
        sessions.insert(id.to_string(), Session::default());
        true
    }

    /// Whether `id` is a live session.
    pub(crate) fn contains(&self, id: &str) -> bool {
        self.lock().contains_key(id)
    }

    /// The number of live sessions.
    pub(crate) fn len(&self) -> usize {
        self.lock().len()
    }

    /// Terminate `id`, dropping its queue. Returns `false` if it was not live. Any
    /// open stream for it observes [`Poll::Gone`] and ends.
    pub(crate) fn remove(&self, id: &str) -> bool {
        let removed = self.lock().remove(id).is_some();
        if removed {
            self.signal.notify_all();
        }
        removed
    }

    /// Queue one server→client JSON-RPC message for `id`. Returns `false` if there is
    /// no such session. The message is queued whether or not a stream is currently
    /// open — an offline client picks it up when it reconnects.
    pub(crate) fn enqueue(&self, id: &str, message: &str) -> bool {
        let mut sessions = self.lock();
        let Some(session) = sessions.get_mut(id) else {
            return false;
        };
        session.next_event_id += 1;
        let event_id = session.next_event_id;
        session.pending.push_back((event_id, message.to_string()));
        drop(sessions);
        self.signal.notify_all();
        true
    }

    /// Queue `message` for every live session. Returns how many sessions received it.
    pub(crate) fn broadcast(&self, message: &str) -> usize {
        let mut sessions = self.lock();
        let mut count = 0usize;
        for session in sessions.values_mut() {
            session.next_event_id += 1;
            let event_id = session.next_event_id;
            session.pending.push_back((event_id, message.to_string()));
            count += 1;
        }
        drop(sessions);
        if count > 0 {
            self.signal.notify_all();
        }
        count
    }

    /// Claim the single SSE stream slot for `id` and return the events to replay
    /// first — everything still in the ring with an id greater than `last_event_id`.
    /// `None` (no `Last-Event-ID` header) replays nothing.
    pub(crate) fn open_stream(
        &self,
        id: &str,
        last_event_id: Option<u64>,
    ) -> Result<Vec<(u64, String)>, StreamError> {
        let mut sessions = self.lock();
        let session = sessions.get_mut(id).ok_or(StreamError::NoSession)?;
        if session.stream_open {
            return Err(StreamError::AlreadyOpen);
        }
        session.stream_open = true;
        let replay = match last_event_id {
            Some(after) => session
                .delivered
                .iter()
                .filter(|(event_id, _)| *event_id > after)
                .cloned()
                .collect(),
            None => Vec::new(),
        };
        Ok(replay)
    }

    /// Release the stream slot for `id` (a no-op if the session is already gone).
    pub(crate) fn close_stream(&self, id: &str) {
        if let Some(session) = self.lock().get_mut(id) {
            session.stream_open = false;
        }
    }

    /// Wait up to `keepalive` for queued messages on `id`, moving whatever it returns
    /// into the replay ring (so a client that dies mid-write can resume from its
    /// `Last-Event-ID`).
    pub(crate) fn poll(&self, id: &str, keepalive: Duration) -> Poll {
        let mut sessions = self.lock();
        loop {
            let Some(session) = sessions.get_mut(id) else {
                return Poll::Gone;
            };
            if !session.pending.is_empty() {
                let events: Vec<(u64, String)> = session.pending.drain(..).collect();
                for event in &events {
                    session.delivered.push_back(event.clone());
                    while session.delivered.len() > self.replay_capacity {
                        session.delivered.pop_front();
                    }
                }
                return Poll::Events(events);
            }
            let (guard, timeout) = self
                .signal
                .wait_timeout(sessions, keepalive)
                .unwrap_or_else(|e| e.into_inner());
            sessions = guard;
            if timeout.timed_out() {
                // Re-check liveness before reporting idle: the session may have been
                // terminated while we waited.
                return if sessions.contains_key(id) {
                    Poll::Idle
                } else {
                    Poll::Gone
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK: Duration = Duration::from_millis(20);

    #[test]
    fn sessions_are_created_found_and_removed() {
        let registry = SessionRegistry::new(8);
        assert_eq!(registry.len(), 0);
        assert!(registry.create("s1", 0));
        assert!(!registry.create("s1", 0), "a duplicate id is refused");
        assert!(registry.contains("s1"));
        assert!(!registry.contains("s2"));
        assert_eq!(registry.len(), 1);
        assert!(registry.remove("s1"));
        assert!(!registry.remove("s1"));
        assert!(!registry.contains("s1"));
    }

    #[test]
    fn queued_messages_come_back_with_monotonic_event_ids() {
        let registry = SessionRegistry::new(8);
        registry.create("s1", 0);
        assert!(registry.enqueue("s1", "one"));
        assert!(registry.enqueue("s1", "two"));
        assert!(!registry.enqueue("missing", "x"), "no session, no delivery");
        match registry.poll("s1", TICK) {
            Poll::Events(events) => assert_eq!(
                events,
                vec![(1, "one".to_string()), (2, "two".to_string())],
                "ids start at 1 and increase"
            ),
            other => panic!("expected the two queued events, got {:?}", other),
        }
    }

    #[test]
    fn an_idle_session_times_out_into_a_keepalive_and_a_dead_one_ends_the_stream() {
        let registry = SessionRegistry::new(8);
        registry.create("s1", 0);
        assert_eq!(registry.poll("s1", TICK), Poll::Idle);
        assert_eq!(registry.poll("gone", TICK), Poll::Gone);
        registry.remove("s1");
        assert_eq!(registry.poll("s1", TICK), Poll::Gone);
    }

    #[test]
    fn broadcast_reaches_every_live_session() {
        let registry = SessionRegistry::new(8);
        registry.create("a", 0);
        registry.create("b", 0);
        assert_eq!(registry.broadcast("hello"), 2);
        for id in ["a", "b"] {
            match registry.poll(id, TICK) {
                Poll::Events(events) => assert_eq!(events, vec![(1, "hello".to_string())]),
                other => panic!("{} missed the broadcast: {:?}", id, other),
            }
        }
        registry.remove("a");
        registry.remove("b");
        assert_eq!(registry.broadcast("nobody"), 0);
    }

    #[test]
    fn only_one_stream_may_hold_a_session_at_a_time() {
        let registry = SessionRegistry::new(8);
        registry.create("s1", 0);
        assert_eq!(registry.open_stream("s1", None), Ok(Vec::new()));
        assert_eq!(registry.open_stream("s1", None), Err(StreamError::AlreadyOpen));
        registry.close_stream("s1");
        assert_eq!(registry.open_stream("s1", None), Ok(Vec::new()));
        assert_eq!(registry.open_stream("nope", None), Err(StreamError::NoSession));
        // Closing an already-terminated session is a no-op, not a panic.
        registry.remove("s1");
        registry.close_stream("s1");
    }

    #[test]
    fn last_event_id_replays_only_the_events_after_it() {
        let registry = SessionRegistry::new(8);
        registry.create("s1", 0);
        registry.open_stream("s1", None).unwrap();
        registry.enqueue("s1", "one");
        registry.enqueue("s1", "two");
        registry.enqueue("s1", "three");
        // Draining moves them into the replay ring.
        assert!(matches!(registry.poll("s1", TICK), Poll::Events(events) if events.len() == 3));
        registry.close_stream("s1");

        let replay = registry.open_stream("s1", Some(1)).unwrap();
        assert_eq!(
            replay,
            vec![(2, "two".to_string()), (3, "three".to_string())],
            "resumption skips what the client already saw"
        );
        registry.close_stream("s1");
        assert_eq!(registry.open_stream("s1", Some(3)).unwrap(), Vec::new());
    }

    #[test]
    fn the_replay_ring_is_bounded_and_drops_the_oldest() {
        let registry = SessionRegistry::new(2);
        registry.create("s1", 0);
        for message in ["one", "two", "three"] {
            registry.enqueue("s1", message);
        }
        assert!(matches!(registry.poll("s1", TICK), Poll::Events(events) if events.len() == 3));
        let replay = registry.open_stream("s1", Some(0)).unwrap();
        assert_eq!(
            replay,
            vec![(2, "two".to_string()), (3, "three".to_string())],
            "only the last two survive a capacity-2 ring"
        );
    }

    #[test]
    fn a_waiting_stream_wakes_as_soon_as_a_message_is_queued() {
        let registry = std::sync::Arc::new(SessionRegistry::new(8));
        registry.create("s1", 0);
        let writer = std::sync::Arc::clone(&registry);
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            writer.enqueue("s1", "late");
        });
        // A generous timeout: the assertion is that the *event* arrives, not Idle.
        match registry.poll("s1", Duration::from_secs(5)) {
            Poll::Events(events) => assert_eq!(events, vec![(1, "late".to_string())]),
            other => panic!("expected the late event, got {:?}", other),
        }
        handle.join().unwrap();
    }
}
