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

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;

    use crate::Descriptor;

    use super::*;

    #[test]
    fn test_serialize_optional_datetime() {
        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        struct Out {
            #[serde(rename = "messageTimestamp")]
            timestamp: String,
        }
        let now = Utc::now();
        let d = Descriptor {
            timestamp: Some(now.clone()),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&d).unwrap();
        let deserialized: Out = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.timestamp,
            now.to_rfc3339_opts(SecondsFormat::Micros, true)
        );
    }
}
