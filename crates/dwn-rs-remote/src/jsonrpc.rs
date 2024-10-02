use serde::{Deserialize, Serialize};

use crate::JSONRpcError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum ID {
    String(String),
    Number(i64),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionRequest {
    id: ID,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Request {
    jsonrpc: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<ID>,
    method: String,
    params: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subscription: Option<SubscriptionRequest>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Result<T: Serialize> {
    result: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Error {
    error: JSONRpcError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum ResultError<T: Serialize> {
    Result(Result<T>),
    Error(Error),
}

// Define the JSONRPCResponse struct
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Response<T: Serialize> {
    jsonrpc: Version,
    id: ID,
    #[serde(flatten)]
    result: ResultError<T>,
}

#[cfg(test)]
mod test {
    use crate::JSONRpcErrorCodes;

    use super::*;

    #[test]
    fn test_request() {
        #[derive(Debug, PartialEq)]
        struct TestCase {
            request: Request,
            expected: &'static str,
        }

        // Define your test cases using the TestCase struct
        let test_cases = vec![
            TestCase {
                request: Request {
                    jsonrpc: Version::V2,
                    id: Some(ID::Number(1)),
                    method: "test".to_string(),
                    params: Some(vec!["param1".to_string(), "param2".to_string()]),
                    subscription: None,
                },
                expected: r#"{"jsonrpc":"2.0","id":1,"method":"test","params":["param1","param2"]}"#,
            },
            TestCase {
                request: Request {
                    jsonrpc: Version::V2,
                    id: None,
                    method: "test".to_string(),
                    params: Some(vec!["param1".to_string(), "param2".to_string()]),
                    subscription: None,
                },
                expected: r#"{"jsonrpc":"2.0","method":"test","params":["param1","param2"]}"#,
            },
            TestCase {
                request: Request {
                    jsonrpc: Version::V2,
                    id: Some(ID::String("1".to_string())),
                    method: "test".to_string(),
                    params: None,
                    subscription: None,
                },
                expected: r#"{"jsonrpc":"2.0","id":"1","method":"test"}"#,
            },
            TestCase {
                request: Request {
                    jsonrpc: Version::V2,
                    id: Some(ID::Number(1)),
                    method: "test".to_string(),
                    params: None,
                    subscription: Some(SubscriptionRequest { id: ID::Number(1) }),
                },
                expected: r#"{"jsonrpc":"2.0","id":1,"method":"test","subscription":{"id":1}}"#,
            },
        ];

        for test_case in test_cases {
            let serialized = serde_json::to_string(&test_case.request).unwrap();
            assert_eq!(
                serialized, test_case.expected,
                "Mismatch for test case {:?}",
                test_case
            );

            let deserialized: Request = serde_json::from_str(&serialized).unwrap();
            assert_eq!(
                test_case.request, deserialized,
                "Deserialization mismatch for {}",
                serialized
            );
        }
    }

    #[test]
    fn test_response() {
        #[derive(Debug, PartialEq)]
        struct TestCase {
            response: Response<String>,
            expected: &'static str,
        }

        let test_cases = vec![
            TestCase {
                response: Response {
                    jsonrpc: Version::V2,
                    id: ID::Number(1),
                    result: ResultError::Result(Result {
                        result: "test".to_string(),
                    }),
                },
                expected: r#"{"jsonrpc":"2.0","id":1,"result":"test"}"#,
            },
            TestCase {
                response: Response {
                    jsonrpc: Version::V2,
                    id: ID::Number(1),
                    result: ResultError::Error(Error {
                        error: JSONRpcError {
                            error: JSONRpcErrorCodes::InvalidRequest,
                            data: None,
                        },
                    }),
                },
                expected: r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#,
            },
        ];

        for test_case in test_cases {
            let serialized = serde_json::to_string(&test_case.response).unwrap();
            assert_eq!(
                serialized, test_case.expected,
                "Mismatch for test case {:?}",
                test_case
            );

            let deserialized: Response<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(
                test_case.response, deserialized,
                "Deserialization mismatch for {}",
                serialized
            );
        }
    }
}
