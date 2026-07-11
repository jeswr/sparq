// AUTHORED-BY Claude Opus 4.8
//! The ActivityStreams 2.0 notification builder.
//!
//! A Solid notification is a JSON-LD ActivityStreams 2.0 object carrying the
//! notifications context, an activity `type` (Update/Create/Delete/Add/Remove), the changed
//! resource as `object`, and a `published` timestamp (per the
//! [Solid Notifications Protocol](https://solidproject.org/TR/notifications-protocol) and
//! [WebSocketChannel2023](https://solid.github.io/notifications/websocket-channel-2023)).
//!
//! This is a **protocol message, not pod RDF** — so it is built as a plain serde value and
//! `serde_json::to_string`'d, NOT routed through the RDF (oxrdf) serialisers. That is the correct
//! exemption: the message shape is defined by the notifications spec, not by stored triples.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

/// The `@context` the Solid Notifications Protocol mandates for notification bodies
/// (`https://www.w3.org/ns/solid/notifications-context/v1`). We additionally include the
/// ActivityStreams 2.0 context so a plain AS2 consumer can interpret `type`/`object`/`published`.
pub const NOTIFICATIONS_CONTEXT: &str = "https://www.w3.org/ns/solid/notifications-context/v1";
/// The ActivityStreams 2.0 context.
pub const AS2_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";

/// The activity type of a change, mapped onto the AS2 verb the notification carries.
///
/// The mapping from an LDP mutation to an activity (the make-the-call choice, documented here):
/// - a resource PUT/PATCH that REPLACED an existing representation ⇒ `Update`,
/// - a resource PUT/PATCH/POST that CREATED a new representation    ⇒ `Create`,
/// - a DELETE                                                       ⇒ `Delete`,
/// - on the PARENT container, a child create                        ⇒ `Add` (membership grew),
/// - on the PARENT container, a child delete                        ⇒ `Remove` (membership shrank).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityType {
    Create,
    Update,
    Delete,
    Add,
    Remove,
}

impl ActivityType {
    /// The AS2 type IRI fragment used in the notification body's `type`.
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityType::Create => "Create",
            ActivityType::Update => "Update",
            ActivityType::Delete => "Delete",
            ActivityType::Add => "Add",
            ActivityType::Remove => "Remove",
        }
    }
}

/// Build the JSON-LD AS2.0 notification body for a change to `resource`.
///
/// `target` is set only for membership activities (`Add`/`Remove`) — per AS2, `Add`/`Remove` carry
/// the collection (the container) as `target` and the contained resource as `object`. For
/// `Create`/`Update`/`Delete` the `object` IS the changed resource and there is no `target`.
///
/// The `published` value is an RFC 3339 / ISO-8601 UTC timestamp derived from the wall clock
/// (built without a date-library dependency — see [`rfc3339_utc`]).
pub fn build_notification(
    activity: ActivityType,
    resource: &str,
    target: Option<&str>,
) -> serde_json::Value {
    let published = rfc3339_utc(now_epoch_secs());
    let mut body = json!({
        "@context": [NOTIFICATIONS_CONTEXT, AS2_CONTEXT],
        "type": activity.as_str(),
        "object": resource,
        "published": published,
    });
    if let Some(t) = target {
        // Membership change: the container is the `target` collection.
        body["target"] = json!(t);
    }
    body
}

/// Build + serialise the notification body to a JSON string (the WS text frame payload).
pub fn build_notification_string(
    activity: ActivityType,
    resource: &str,
    target: Option<&str>,
) -> String {
    // `to_string` on a serde_json::Value cannot fail; the value is always serialisable.
    build_notification(activity, resource, target).to_string()
}

/// The current wall-clock time as epoch seconds (0 if the clock is before the epoch — never panics).
fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Format epoch seconds as an RFC 3339 / ISO-8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`).
///
/// Implemented from first principles (a civil-date conversion via the days-from-epoch algorithm)
/// rather than pulling in `chrono`/`time` — the notification only needs a correct UTC second-grained
/// stamp, and adding a date crate to an experimental server for one format call is not warranted
/// (the standing make-the-call rule). The algorithm is Howard Hinnant's well-known
/// `civil_from_days` (public-domain), correct across the proleptic Gregorian calendar.
pub fn rfc3339_utc(epoch_secs: u64) -> String {
    let secs_of_day = epoch_secs % 86_400;
    let days = (epoch_secs / 86_400) as i64;
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    // civil_from_days: days since 1970-01-01 -> (year, month, day). Shift the epoch to an internal
    // era starting 0000-03-01 so leap handling is uniform.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_known_epochs() {
        // 0 -> the epoch instant.
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        // 1_000_000_000 is a well-known epoch value.
        assert_eq!(rfc3339_utc(1_000_000_000), "2001-09-09T01:46:40Z");
        // A leap-year date (2000 is a leap year; 2000-02-29 exists).
        // 2000-02-29T12:00:00Z = 951_825_600.
        assert_eq!(rfc3339_utc(951_825_600), "2000-02-29T12:00:00Z");
    }

    #[test]
    fn create_notification_shape() {
        let v = build_notification(ActivityType::Create, "https://pod.example/a", None);
        let ctx = v["@context"].as_array().expect("@context is an array");
        assert!(ctx.iter().any(|c| c == NOTIFICATIONS_CONTEXT));
        assert!(ctx.iter().any(|c| c == AS2_CONTEXT));
        assert_eq!(v["type"], "Create");
        assert_eq!(v["object"], "https://pod.example/a");
        assert!(v.get("target").is_none(), "Create carries no target");
        let published = v["published"].as_str().expect("published is a string");
        assert!(published.ends_with('Z'), "published is UTC: {published}");
    }

    #[test]
    fn add_notification_carries_container_target() {
        let v = build_notification(
            ActivityType::Add,
            "https://pod.example/c/child",
            Some("https://pod.example/c/"),
        );
        assert_eq!(v["type"], "Add");
        assert_eq!(v["object"], "https://pod.example/c/child");
        // AS2 Add: the container collection is the `target`.
        assert_eq!(v["target"], "https://pod.example/c/");
    }

    #[test]
    fn each_activity_serialises_to_its_verb() {
        for (a, want) in [
            (ActivityType::Create, "Create"),
            (ActivityType::Update, "Update"),
            (ActivityType::Delete, "Delete"),
            (ActivityType::Add, "Add"),
            (ActivityType::Remove, "Remove"),
        ] {
            assert_eq!(a.as_str(), want);
            let s = build_notification_string(a, "https://pod.example/x", None);
            assert!(s.contains(&format!("\"type\":\"{want}\"")), "{s}");
        }
    }
}
