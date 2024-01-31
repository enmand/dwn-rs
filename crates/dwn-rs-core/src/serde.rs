use chrono::{DateTime, Utc};

pub fn serialize_optional_datetime<S>(
    date: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match date {
        Some(date) => {
            serializer.serialize_str(&date.to_rfc3339_opts(chrono::SecondsFormat::Micros, true))
        }
        None => serializer.serialize_none(),
    }
}
