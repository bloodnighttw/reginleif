use chrono::{DateTime, Local};
use serde::{de, Deserialize, Deserializer, Serializer};
use serde_json::Value;

/// This function is intend to be used with serde's `deserialize_with` attribute.
/// Convert a string to a DateTime<Local>.
/// The string should be in RFC3339 format.
pub fn string_to_local<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Local>, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(str) => {
            let t = str.as_str();
            let datetime = DateTime::parse_from_rfc3339(t).expect("Failed to parse time");
            datetime.with_timezone(&Local)
        }
        _ => return Err(de::Error::custom("wrong type")),
    })
}

/// This function is intend to be used with serde's `serialize_with` attribute.
/// Convert a DateTime<Local> to a string.
/// The string will be in RFC3339 format.
pub fn local_to_string<S>(x: &DateTime<Local>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(x.to_rfc3339().as_str())
}
