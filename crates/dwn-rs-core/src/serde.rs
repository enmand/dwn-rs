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

pub mod identifier {
    use serde::{Deserialize, Deserializer, Serializer};
    use web5::dids::identifier::Identifier;

    pub fn serialize<S>(id: &Option<Identifier>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match id {
            Some(id) => serializer.serialize_str(&id.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Identifier>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Some(
            Identifier::parse(&s).map_err(serde::de::Error::custom)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;

    use crate::GenericDescriptor;

    use super::*;

    #[test]
    fn test_serialize_optional_datetime() {
        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        struct Out {
            #[serde(rename = "messageTimestamp")]
            timestamp: String,
        }
        let now = Utc::now();
        let d = GenericDescriptor {
            timestamp: Some(now),
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
