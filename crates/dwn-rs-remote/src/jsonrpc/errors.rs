use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JSONRpcError {
    pub code: JSONRpcErrorCodes,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Display for JSONRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(data) = &self.data {
            write!(f, "{:?}: {} ({})", self.code, self.message, data)
        } else {
            write!(f, "{:?}: {}", self.code, self.message)
        }
    }
}

impl Error for JSONRpcError {}

impl From<reqwest::Error> for JSONRpcError {
    fn from(err: reqwest::Error) -> Self {
        JSONRpcError {
            code: JSONRpcErrorCodes::InternalError,
            message: err.to_string(),
            data: None,
        }
    }
}

impl From<serde_json::Error> for JSONRpcError {
    fn from(err: serde_json::Error) -> Self {
        JSONRpcError {
            code: JSONRpcErrorCodes::InternalError,
            message: err.to_string(),
            data: None,
        }
    }
}

impl From<ulid::MonotonicError> for JSONRpcError {
    fn from(err: ulid::MonotonicError) -> Self {
        JSONRpcError {
            code: JSONRpcErrorCodes::InternalError,
            message: err.to_string(),
            data: None,
        }
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_error_display() {
        let error = JSONRpcError {
            code: JSONRpcErrorCodes::InvalidRequest,
            message: "Invalid Request".to_string(),
            data: None,
        };
        assert_eq!(error.to_string(), "InvalidRequest: Invalid Request");

        let error = JSONRpcError {
            code: JSONRpcErrorCodes::InvalidRequest,
            message: "Invalid Request".to_string(),
            data: Some(serde_json::json!({ "error": "test" })),
        };
        assert_eq!(
            error.to_string(),
            "InvalidRequest: Invalid Request ({\"error\":\"test\"})"
        );
    }

    #[tokio::test]
    async fn test_jsonrpc_error_from() {
        let err = reqwest::get("bad address").await.unwrap_err();
        let errstr = err.to_string();
        let jsonrpc_err: JSONRpcError = err.into();
        assert_eq!(jsonrpc_err.code, JSONRpcErrorCodes::InternalError);
        assert_eq!(jsonrpc_err.message, errstr);

        let err = serde_json::from_str::<&str>("bad json").unwrap_err();
        let errstr = err.to_string();
        let jsonrpc_err: JSONRpcError = err.into();
        assert_eq!(jsonrpc_err.code, JSONRpcErrorCodes::InternalError);
        assert_eq!(jsonrpc_err.message, errstr);
    }

    #[test]
    fn test_jsonrpc_errorcodes_serialization() {
        let codes: Vec<(JSONRpcErrorCodes, &'static str)> = vec![
            (
                JSONRpcErrorCodes::InvalidRequest,
                r#"{"code":-32600,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::MethodNotFound,
                r#"{"code":-32601,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::InvalidParams,
                r#"{"code":-32602,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::InternalError,
                r#"{"code":-32603,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::ParseError,
                r#"{"code":-32700,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::BadRequest,
                r#"{"code":-50400,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::Unauthorized,
                r#"{"code":-50401,"message":"error"}"#,
            ),
            (
                JSONRpcErrorCodes::Forbidden,
                r#"{"code":-50403,"message":"error"}"#,
            ),
        ];

        for (code, json) in codes {
            let err: JSONRpcError = serde_json::from_str(json).unwrap();

            let expected = JSONRpcError {
                code,
                message: "error".to_string(),
                data: None,
            };

            assert_eq!(err, expected);
        }
    }

    #[test]
    fn test_jsonrpc_serialization() {
        let error = JSONRpcError {
            code: JSONRpcErrorCodes::InvalidRequest,
            message: "error".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, r#"{"code":-32600,"message":"error"}"#);

        let error: JSONRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(error.code, JSONRpcErrorCodes::InvalidRequest);
        assert_eq!(error.data, None);
    }
}
