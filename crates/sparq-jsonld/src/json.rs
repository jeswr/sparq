//! The minimal, dependency-free JSON value model (`Json` AST).
//!
//! [OPUS-4.8] (sq-oy1f.23) Moved **verbatim** out of `sparq-engine`'s JSON-LD writer
//! (`serialize/compact.rs`) so the whole document-level JSON-LD 1.1 pipeline can share one
//! JSON value type without pulling in `serde_json`. The move is behaviour-neutral: the
//! engine re-exports this type at its old path (`sparq_engine::serialize::compact::Json`,
//! aliased `JsonLdValue`), so its writer and every downstream consumer keep compiling and
//! emitting byte-identical output.
//!
//! Object members preserve **insertion order** (the writer is order-deterministic), so an
//! object is a `Vec<(String, Json)>` rather than a hash map.

use std::fmt::Write as _;

/// A minimal JSON value used as the working representation for the JSON-LD pipeline. Object
/// members preserve insertion order so the emitted document is deterministic.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    /// A JSON string.
    Str(String),
    /// A pre-rendered JSON scalar token (`true`, `false`, a number). Stored verbatim so
    /// the writer's lossless native-coercion text is preserved byte-for-byte.
    Raw(String),
    /// A JSON array.
    Arr(Vec<Json>),
    /// A JSON object, members in insertion order.
    Obj(Vec<(String, Json)>),
}

impl Default for Json {
    /// The default JSON value is an empty object — the natural "no context" value for a
    /// processed active context's raw `@context`.
    fn default() -> Json {
        Json::Obj(Vec::new())
    }
}

impl Json {
    /// Constructs an empty JSON object.
    pub fn obj() -> Json {
        Json::Obj(Vec::new())
    }

    /// Inserts/overwrites a member, preserving first-insertion position for an existing key.
    /// A no-op on a non-object value.
    pub fn set(&mut self, key: &str, val: Json) {
        if let Json::Obj(members) = self {
            if let Some(slot) = members.iter_mut().find(|(k, _)| k == key) {
                slot.1 = val;
            } else {
                members.push((key.to_string(), val));
            }
        }
    }

    /// Borrows the value of member `key`, or `None` if this is not an object or the key is
    /// absent.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Borrows the inner string of a [`Json::Str`], or `None` for any other variant.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// True iff this is a [`Json::Obj`].
    pub fn is_obj(&self) -> bool {
        matches!(self, Json::Obj(_))
    }

    /// Serialises this value as canonical minified JSON into `out`.
    pub fn write(&self, out: &mut String) {
        match self {
            Json::Str(s) => {
                out.push('"');
                json_escape(s, out);
                out.push('"');
            }
            Json::Raw(r) => out.push_str(r),
            Json::Arr(items) => {
                out.push('[');
                for (i, it) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    it.write(out);
                }
                out.push(']');
            }
            Json::Obj(members) => {
                out.push('{');
                for (i, (k, v)) in members.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push('"');
                    json_escape(k, out);
                    out.push_str("\":");
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

/// Escapes a string as a JSON string body (without the surrounding quotes) per RFC 8259:
/// the two mandatory escapes (`"`, `\`), the short escapes for the common control chars,
/// and `\u00XX` for the remaining C0 controls. Everything else (including non-ASCII, which
/// JSON permits raw in UTF-8) passes through verbatim.
fn json_escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty_object() {
        assert_eq!(Json::default(), Json::Obj(Vec::new()));
        assert!(Json::default().is_obj());
    }

    #[test]
    fn obj_constructs_empty_object() {
        let o = Json::obj();
        assert_eq!(o, Json::Obj(Vec::new()));
        assert!(o.is_obj());
    }

    #[test]
    fn set_inserts_and_preserves_position_then_overwrites() {
        let mut o = Json::obj();
        o.set("a", Json::Str("1".into()));
        o.set("b", Json::Str("2".into()));
        o.set("a", Json::Str("3".into())); // overwrite keeps first position
        assert_eq!(
            o,
            Json::Obj(vec![
                ("a".into(), Json::Str("3".into())),
                ("b".into(), Json::Str("2".into())),
            ])
        );
        // set() on a non-object is a no-op.
        let mut arr = Json::Arr(vec![]);
        arr.set("x", Json::Raw("true".into()));
        assert_eq!(arr, Json::Arr(vec![]));
    }

    #[test]
    fn get_borrows_member_or_none() {
        let mut o = Json::obj();
        o.set("k", Json::Raw("42".into()));
        assert_eq!(o.get("k"), Some(&Json::Raw("42".into())));
        assert_eq!(o.get("missing"), None);
        // get() on a non-object is None.
        assert_eq!(Json::Str("x".into()).get("k"), None);
    }

    #[test]
    fn as_str_only_for_strings() {
        assert_eq!(Json::Str("hi".into()).as_str(), Some("hi"));
        assert_eq!(Json::Raw("1".into()).as_str(), None);
        assert_eq!(Json::obj().as_str(), None);
    }

    #[test]
    fn is_obj_discriminates() {
        assert!(Json::obj().is_obj());
        assert!(!Json::Arr(vec![]).is_obj());
        assert!(!Json::Str("x".into()).is_obj());
    }

    #[test]
    fn write_emits_canonical_minified_json_with_escaping() {
        let mut o = Json::obj();
        o.set("s", Json::Str("a\"b\\c\n\t".into()));
        o.set("n", Json::Raw("3.5".into()));
        o.set(
            "a",
            Json::Arr(vec![Json::Raw("true".into()), Json::Str("x".into())]),
        );
        let mut out = String::new();
        o.write(&mut out);
        assert_eq!(out, "{\"s\":\"a\\\"b\\\\c\\n\\t\",\"n\":3.5,\"a\":[true,\"x\"]}");
    }

    #[test]
    fn write_escapes_c0_controls_as_lower_hex() {
        let mut out = String::new();
        Json::Str("\u{01}\u{1f}".into()).write(&mut out);
        // A quote, the two C0 controls as `\u00XX` lower-hex, then a quote.
        assert_eq!(out, "\"\\u0001\\u001f\"");
    }
}
