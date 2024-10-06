use std::{future::Future, pin::Pin};

use bytes::Bytes;
use dwn_rs_core::Response as DWNResponse;
use futures_core::{stream::BoxStream, TryStream};
use futures_util::TryStreamExt;
use http::header;
use serde_json::json;
use tower::Service;
use tracing::trace;

use crate::jsonrpc::ResultError;

use super::{JSONRpcError, JSONRpcErrorCodes, Request, Response};

pub const USER_AGENT: &str = concat!(
    "rpc-",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "(",
    env!("CARGO_CRATE_NAME"),
    ")"
);

pub struct HTTPTransport {
    url: String,
    client: reqwest::Client,
}

impl<S> Service<(Request, Option<S>)> for HTTPTransport
where
    S: TryStream + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Bytes: From<S::Ok>,
{
    type Response = Response<(DWNResponse, BoxStream<'static, Result<Bytes, JSONRpcError>>)>;
    type Error = JSONRpcError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, request: (Request, Option<S>)) -> Self::Future {
        let url = self.url.clone();
        let mut client = self.client.clone();

        Box::pin(async move {
            let stream = request.1;

            let mut rb = client
                .clone()
                .request(http::method::Method::POST, url)
                .header("dwn-request", json!(request.0).to_string());

            if let Some(data) = stream {
                let (lw, _) = data.size_hint();

                rb = rb
                    .header(header::CONTENT_TYPE, "application/octet-stream")
                    .header(header::TRANSFER_ENCODING, "chunked")
                    .header(header::CONTENT_LENGTH, lw.to_string())
                    .body(reqwest::Body::wrap_stream(data));
            }

            let req = rb.build()?;

            let res = client.call(req).await?;

            trace!(?res, "Received response");

            let resp = match res.headers().get("dwn-response") {
                Some(h) => {
                    let resp = serde_json::from_slice::<Response<DWNResponse>>(h.as_bytes())?;
                    let body = Box::pin(
                        res.bytes_stream()
                            .map_err(JSONRpcError::from)
                            .map_ok(|b| b)
                            .into_stream(),
                    )
                        as BoxStream<'static, Result<Bytes, JSONRpcError>>;

                    (resp, body)
                }
                None => {
                    let body = res.bytes().await?;

                    let resp: Response<DWNResponse> = serde_json::from_slice(&body)?;
                    trace!(?resp, "Response in body");
                    let empty =
                        Box::pin(futures_util::stream::empty::<Result<Bytes, JSONRpcError>>())
                            as BoxStream<'static, Result<Bytes, JSONRpcError>>;

                    (resp, empty)
                }
            };

            let msg = match resp.0.result {
                ResultError::Result(m) => Ok(m.reply),
                ResultError::Error(e) => Err(JSONRpcError::from(e)),
            }?;
            let body = resp.1;

            Ok(Response::new_v2(resp.0.id, (msg, body)))
        })
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.client.poll_ready(cx).map_err(|e| JSONRpcError {
            code: JSONRpcErrorCodes::InternalError,
            message: e.to_string(),
            data: None,
        })
    }
}

impl HTTPTransport {
    pub fn new(uri: String) -> Result<Self, JSONRpcError> {
        let c = reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .default_headers(header::HeaderMap::from_iter(vec![(
                header::CONTENT_TYPE,
                "application/json-rpc".parse().unwrap(),
            )]))
            .deflate(true)
            .gzip(true)
            .build()
            .map_err(|e| JSONRpcError {
                code: JSONRpcErrorCodes::InternalError,
                message: e.to_string(),
                data: None,
            });

        Ok(Self {
            url: uri,
            client: c?,
        })
    }
}
