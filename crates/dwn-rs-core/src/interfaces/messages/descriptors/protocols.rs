use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ssi_dids_core::DIDBuf;

use crate::descriptors::MessageDescriptor;
use crate::interfaces::messages::descriptors::{CONFIGURE, PROTOCOLS, QUERY};
use crate::{protocols, Message};
use dwn_rs_message_derive::descriptor;

use super::RecordsWriteDescriptor;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ConfigureParameters {
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub definition: protocols::Definition,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<Message<RecordsWriteDescriptor>>,
}

#[descriptor(interface = PROTOCOLS, method = CONFIGURE, fields = crate::fields::AuthorizationDelegatedGrantFields, parameters = ConfigureParameters)]
pub struct ConfigureDescriptor {
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub definition: protocols::Definition,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryParameters {
    pub filter: Option<QueryFilterParameters>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
}

#[descriptor(interface = PROTOCOLS , method = QUERY, fields = crate::auth::Authorization, parameters = QueryParameters)]
pub struct QueryDescriptor {
    #[serde(rename = "message_timestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: Option<QueryFilter>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryFilterParameters {
    pub protocol: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct QueryFilter {
    pub protocol: Option<String>,
    pub recipient: Option<DIDBuf>,
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::protocols::ActionWho;

    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use ssi_jwk::JWK;

    #[test]
    fn test_configure_descriptor() {
        let message_timestamp = Utc::now();
        let definition = protocols::Definition {
            protocol: "example".to_string(),
            published: true,
            types: BTreeMap::new(),
            structure: BTreeMap::new(),
        };
        let descriptor = ConfigureDescriptor {
            message_timestamp,
            definition,
        };
        let json = json!({
            "messageTimestamp": message_timestamp,
            "definition": {
                "protocol": "example",
                "published": true,
                "types": {},
                "structure": {},
            },
            "interface": PROTOCOLS,
            "method": CONFIGURE,
        });
        assert_eq!(serde_json::to_value(&descriptor).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<ConfigureDescriptor>(json).unwrap(),
            descriptor
        );
    }

    #[test]
    fn test_protocol_definition() {
        let protocol = "example".to_string();
        let published = true;
        let types = BTreeMap::new();
        let structure = BTreeMap::new();
        let definition = protocols::Definition {
            protocol: protocol.clone(),
            published,
            types,
            structure,
        };
        let json = json!({
            "protocol": protocol,
            "published": published,
            "types": {},
            "structure": {},
        });
        assert_eq!(serde_json::to_value(&definition).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<protocols::Definition>(json).unwrap(),
            definition
        );
    }

    #[test]
    fn test_protocol_type() {
        let schema = Some("schema".to_string());
        let data_formats = Some(vec!["format".to_string()]);
        let protocol_type = protocols::Type {
            schema: schema.clone(),
            data_formats: data_formats.clone(),
        };
        let json = json!({
            "schema": schema,
            "dataFormats": data_formats,
        });
        assert_eq!(serde_json::to_value(&protocol_type).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<protocols::Type>(json).unwrap(),
            protocol_type
        );
    }

    #[test]
    fn test_protocol_rule() {
        let encryption = Some(protocols::PathEncryption {
            root_key_id: "root".to_string(),
            public_key_jwk: JWK::generate_ed25519().unwrap(),
        });
        let actions = vec![protocols::Action::Who(ActionWho {
            who: protocols::Who::Anyone,
            of: None,
            can: vec![protocols::Can::Read],
        })];

        let role = Some(true);
        let size = Some(protocols::Size {
            min: None,
            max: None,
        });
        let tags = Some(protocols::Tags {
            required_tags: vec!["tag".to_string()],
            allow_undefined_tags: Some(true),
            tags: BTreeMap::new(),
        });

        let rules: BTreeMap<String, protocols::RuleSet> = BTreeMap::new();
        let protocol_rule = protocols::RuleSet {
            encryption: encryption.clone(),
            actions: actions.clone(),
            role,
            size: size.clone(),
            tags: tags.clone(),
            rules,
        };

        let json = json!({
            "$encryption": encryption.clone(),
            "$actions": actions,
            "$role": role,
            "$size": size,
            "$tags": tags,
        });

        assert_eq!(serde_json::to_value(&protocol_rule).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<protocols::RuleSet>(json).unwrap(),
            protocol_rule
        );

        let json = json!({
            "$encryption": encryption,
            "$actions": actions,
            "$role": role,
            "$size": size,
            "$tags": tags,
            "key": {},
        });

        let mut rules: BTreeMap<String, protocols::RuleSet> = BTreeMap::new();
        rules.insert("key".to_string(), protocols::RuleSet::default());
        let protocol_rule = protocols::RuleSet {
            encryption,
            actions,
            role,
            size,
            tags,
            rules,
        };

        assert_eq!(serde_json::to_value(&protocol_rule).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<protocols::RuleSet>(json).unwrap(),
            protocol_rule
        );
    }
}
