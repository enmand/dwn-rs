use std::{any::Any, fmt::Debug};

use bytes::Bytes;
use dwn_rs_core::Response as DWNResponse;
use futures_core::{stream::BoxStream, Stream, TryStream};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tower::Service;
use ulid::{Generator, Ulid};

use super::JSONRpcError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ID {
    String(String),
    Number(i64),
}

impl From<Ulid> for ID {
    fn from(ulid: Ulid) -> Self {
        Self::String(ulid.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionRequest {
    id: ID,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    jsonrpc: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<ID>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subscription: Option<SubscriptionRequest>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResultData<T> {
    pub reply: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Error {
    error: JSONRpcError,
}

impl From<Error> for JSONRpcError {
    fn from(error: Error) -> Self {
        error.error
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResultError<T> {
    Result(ResultData<T>),
    Error(Error),
}

// Define the JSONRPCResponse struct
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Response<T> {
    jsonrpc: Version,
    pub id: ID,
    pub result: ResultError<T>,
}

impl<T> Response<T> {
    pub fn new_v2<I>(id: I, reply: T) -> Self
    where
        I: Into<ID>,
    {
        Self {
            jsonrpc: Version::V2,
            id: id.into(),
            result: ResultError::Result(ResultData { reply }),
        }
    }
}

pub struct Client<T: Service<(Request, Option<S>)>, S> {
    ulid: Generator,
    transport: T,
    _phantom: std::marker::PhantomData<S>,
}

impl<T, S> std::fmt::Debug for Client<T, S>
where
    T: Service<(Request, Option<S>)> + Debug,
    S: Stream<Item = Bytes>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("ulid", &self.ulid.type_id())
            .field("transport", &self.transport)
            .finish()
    }
}

impl<T, S> Client<T, S>
where
    T: Service<
        (Request, Option<S>),
        Response = Response<(DWNResponse, BoxStream<'static, Result<Bytes, JSONRpcError>>)>,
        Error = JSONRpcError,
    >,
    S: TryStream + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Bytes: From<S::Ok>,
{
    pub fn new(transport: T) -> Self {
        let ulid = Generator::new();

        Self {
            ulid,
            transport,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn request<P: Serialize + DeserializeOwned>(
        &mut self,
        method: &'static str,
        params: P,
        data: Option<S>,
    ) -> Result<
        Response<(DWNResponse, impl Stream<Item = Result<Bytes, JSONRpcError>>)>,
        JSONRpcError,
    > {
        let id = Some(self.ulid.generate()?.into());

        let jsonrpc = Version::V2;
        let method = method.to_string();

        let request = Request {
            jsonrpc,
            id,
            method,
            params: Some(serde_json::to_value(params)?),
            subscription: None,
        };

        self.transport.call((request, data)).await
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::jsonrpc::JSONRpcErrorCodes;

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
                    params: Some(json!(vec!["param1".to_string(), "param2".to_string()])),
                    subscription: None,
                },
                expected: r#"{"jsonrpc":"2.0","id":1,"method":"test","params":["param1","param2"]}"#,
            },
            TestCase {
                request: Request {
                    jsonrpc: Version::V2,
                    id: None,
                    method: "test".to_string(),
                    params: Some(json!(vec!["param1".to_string(), "param2".to_string()])),
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
                    result: ResultError::Result(ResultData {
                        reply: "test".to_string(),
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
                            code: JSONRpcErrorCodes::InvalidRequest,
                            message: "Invalid Request".to_string(),
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
