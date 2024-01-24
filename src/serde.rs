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

pub fn serialize_datetime<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.to_rfc3339_opts(chrono::SecondsFormat::Micros, true))
}
