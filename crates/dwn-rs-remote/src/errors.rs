use serde::{ser::SerializeMap, Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RemoteError>;

#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("error: {0}")]
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JSONRpcError {
    #[serde(flatten)]
    pub error: JSONRpcErrorCodes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JSONRpcErrorCodes {
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ParseError = -32700,

    BadRequest = -50400,
    Unauthorized = -50401,
    Forbidden = -50403,
}

// Serialize JSONRpcErrorCodes as { "code": number, "message": string }
impl Serialize for JSONRpcErrorCodes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let code = *self as i64;
        let message = match self {
            JSONRpcErrorCodes::InvalidRequest => "Invalid Request",
            JSONRpcErrorCodes::MethodNotFound => "Method Not Found",
            JSONRpcErrorCodes::InvalidParams => "Invalid Params",
            JSONRpcErrorCodes::InternalError => "Internal Error",
            JSONRpcErrorCodes::ParseError => "Parse Error",
            JSONRpcErrorCodes::BadRequest => "Bad Request",
            JSONRpcErrorCodes::Unauthorized => "Unauthorized",
            JSONRpcErrorCodes::Forbidden => "Forbidden",
        };
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("code", &code)?;
        map.serialize_entry("message", message)?;
        map.end()
    }
}

// Deserialize JSONRpcErrorCodes from { "code": number, "message": string }
// or numeric code
impl<'de> Deserialize<'de> for JSONRpcErrorCodes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<JSONRpcErrorCodes, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JSONRpcErrorCodesVisitor;

        impl<'de> serde::de::Visitor<'de> for JSONRpcErrorCodesVisitor {
            type Value = JSONRpcErrorCodes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSONRpcErrorCodes object or a number")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut code = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key.as_str() == "code" {
                        code = Some(map.next_value()?);
                    }
                }
                match code {
                    Some(code) => match code {
                        -32600 => Ok(JSONRpcErrorCodes::InvalidRequest),
                        -32601 => Ok(JSONRpcErrorCodes::MethodNotFound),
                        -32602 => Ok(JSONRpcErrorCodes::InvalidParams),
                        -32603 => Ok(JSONRpcErrorCodes::InternalError),
                        -32700 => Ok(JSONRpcErrorCodes::ParseError),
                        -50400 => Ok(JSONRpcErrorCodes::BadRequest),
                        -50401 => Ok(JSONRpcErrorCodes::Unauthorized),
                        -50403 => Ok(JSONRpcErrorCodes::Forbidden),
                        _ => Err(serde::de::Error::custom(format!("unknown code: {}", code))),
                    },
                    None => Err(serde::de::Error::missing_field("code")),
                }
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    -32600 => Ok(JSONRpcErrorCodes::InvalidRequest),
                    -32601 => Ok(JSONRpcErrorCodes::MethodNotFound),
                    -32602 => Ok(JSONRpcErrorCodes::InvalidParams),
                    -32603 => Ok(JSONRpcErrorCodes::InternalError),
                    -32700 => Ok(JSONRpcErrorCodes::ParseError),
                    -50400 => Ok(JSONRpcErrorCodes::BadRequest),
                    -50401 => Ok(JSONRpcErrorCodes::Unauthorized),
                    -50403 => Ok(JSONRpcErrorCodes::Forbidden),
                    _ => Err(serde::de::Error::custom(format!("unknown code: {}", v))),
                }
            }
        }

        deserializer.deserialize_any(JSONRpcErrorCodesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_errorcodes_serialization() {
        let codes: Vec<(JSONRpcErrorCodes, &'static str)> = vec![
            (
                JSONRpcErrorCodes::InvalidRequest,
                r#"{"code":-32600,"message":"Invalid Request"}"#,
            ),
            (
                JSONRpcErrorCodes::MethodNotFound,
                r#"{"code":-32601,"message":"Method Not Found"}"#,
            ),
            (
                JSONRpcErrorCodes::InvalidParams,
                r#"{"code":-32602,"message":"Invalid Params"}"#,
            ),
            (
                JSONRpcErrorCodes::InternalError,
                r#"{"code":-32603,"message":"Internal Error"}"#,
            ),
            (
                JSONRpcErrorCodes::ParseError,
                r#"{"code":-32700,"message":"Parse Error"}"#,
            ),
            (
                JSONRpcErrorCodes::BadRequest,
                r#"{"code":-50400,"message":"Bad Request"}"#,
            ),
            (
                JSONRpcErrorCodes::Unauthorized,
                r#"{"code":-50401,"message":"Unauthorized"}"#,
            ),
            (
                JSONRpcErrorCodes::Forbidden,
                r#"{"code":-50403,"message":"Forbidden"}"#,
            ),
        ];

        for (code, json) in codes {
            let err: JSONRpcError = serde_json::from_str(json).unwrap();

            let expected = JSONRpcError {
                error: code,
                data: None,
            };

            assert_eq!(err, expected);
        }
    }

    #[test]
    fn test_jsonrpc_serialization() {
        let error = JSONRpcError {
            error: JSONRpcErrorCodes::InvalidRequest,
            data: None,
        };
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, r#"{"code":-32600,"message":"Invalid Request"}"#);

        let error: JSONRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(error.error, JSONRpcErrorCodes::InvalidRequest);
        assert_eq!(error.data, None);
    }
}
