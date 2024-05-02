use serde::Serialize;
use surrealdb::{
    iam::Level,
    opt::auth::{self, Credentials},
};
use thiserror::Error;

#[derive(Serialize, Clone, Debug)]
pub struct Auth {
    #[serde(rename = "user", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "pass", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(rename = "ns")]
    pub namespace: String,
    pub level: Level,
}
impl PartialEq for Auth {
    fn eq(&self, other: &Self) -> bool {
        match (self.level.clone(), other.level.clone()) {
            (Level::No, Level::No) => self.namespace == other.namespace,
            (Level::Root, Level::Root) => {
                self.username == other.username
                    && self.password == other.password
                    && self.namespace == other.namespace
            }
            (Level::Namespace(sns), Level::Namespace(ons)) => {
                self.username == other.username
                    && self.password == other.password
                    && self.namespace == other.namespace
                    && self.namespace == sns
                    && self.namespace == ons
            }
            _ => false,
        }
    }
}

impl Auth {
    // has auth returns true if authenticaiton material (username/password)
    // exists
    pub fn has_auth(&self) -> bool {
        matches!(
            self,
            Auth {
                username: Some(_),
                password: Some(_),
                ..
            }
        )
    }

    pub fn ns(&self) -> &str {
        self.namespace.as_str()
    }

    // parse the connection string for authentication information that can be used to sign in. The
    // connection string, with credentials, is expected to be in the format:
    //
    //   `<proto>://<username>:<password>@<host>:<port>/<namespace>?auth=[root|namespace]`
    //
    // for Namespace-based authentication.
    //
    // Returns the connection string and the credentials.
    pub(crate) fn parse_connstr(connstr: &str) -> Result<(String, Option<Self>), AuthError> {
        let connstr = connstr.trim();
        let parts = url::Url::parse(connstr)?;

        if parts.scheme().is_empty() {
            return Err(AuthError::InvalidConnStr(url::ParseError::EmptyHost));
        }

        let namespace = parts.path().trim_start_matches('/');

        let mut auth = Auth {
            username: None,
            password: None,
            namespace: namespace.to_string(),
            level: Level::No,
        };

        if parts.has_authority() && parts.password().is_some() {
            auth.username = Some(parts.username().to_owned());
            auth.password = Some(parts.password().unwrap_or_default().to_owned());
            auth.level = Level::Root;
        }

        // auth type is defined by the query parameter
        if let Some((_, v)) = parts.query_pairs().find(|(k, _)| k == "auth") {
            match v.as_ref() {
                "root" => {
                    auth.level = Level::Root;
                }
                "namespace" => {
                    auth.level = Level::Namespace(namespace.to_string());
                }
                _ => {
                    return Err(AuthError::InvalidAuthInfo(v.to_string()));
                }
            }
        };

        let connstr = format!(
            "{}://{}",
            parts.scheme(),
            match parts.host() {
                Some(host) => {
                    if let Some(port) = parts.port() {
                        format!("{}:{}", host, port)
                    } else {
                        host.to_string()
                    }
                }
                None => "".to_string(), // e.g. memory:// of file:// will not have a proper host
            },
        );
        Ok((connstr, Some(auth)))
    }
}

impl Credentials<auth::Signin, auth::Jwt> for Auth {}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid connection string: {0}")]
    InvalidConnStr(#[from] url::ParseError),

    #[error("invalid authentication information: {0}")]
    InvalidAuthInfo(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connstr() {
        let connstr = "mem:///ns";
        let (connstr, creds) = Auth::parse_connstr(connstr).expect("parseable");
        assert_eq!(connstr, "mem://");
        assert_eq!(
            creds,
            Some(Auth {
                username: None,
                password: None,
                namespace: "ns".to_string(),
                level: Level::No,
            })
        );

        let connstr = "http://user:pass@localhost:8080/ns";
        let (connstr, creds) = Auth::parse_connstr(connstr).expect("parseable");
        assert_eq!(connstr, "http://localhost:8080");
        assert_eq!(
            creds,
            Some(Auth {
                username: Some("user".to_string()),
                password: Some("pass".to_string()),
                namespace: "ns".to_string(),
                level: Level::Root,
            })
        );

        let connstr = "http://user:password@localhost:8080/ns?auth=root";
        let (connstr, creds) = Auth::parse_connstr(connstr).expect("parsable");
        assert_eq!(connstr, "http://localhost:8080");
        assert_eq!(
            creds,
            Some(Auth {
                username: Some("user".to_string()),
                password: Some("password".to_string()),
                namespace: "ns".to_string(),
                level: Level::Root,
            })
        );

        let connstr = "http://user:password@localhost:8080/ns?auth=namespace";
        let (connstr, creds) = Auth::parse_connstr(connstr).expect("parsable");
        assert_eq!(connstr, "http://localhost:8080");
        assert_eq!(
            creds,
            Some(Auth {
                username: Some("user".to_string()),
                password: Some("password".to_string()),
                namespace: "ns".to_string(),
                level: Level::Namespace("ns".to_string()),
            })
        );
    }
}
