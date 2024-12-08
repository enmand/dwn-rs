use crate::{
    errors::Result as ClientResult,
    jsonrpc::{self, JSONRpcError},
    RemoteError,
};

use bytes::Bytes;
use futures_core::{stream::BoxStream, Stream, TryStream};
use futures_util::StreamExt;
use tower::Service;

use dwn_rs_core::{Message, Response as DWNResponse};

pub struct RemoteDWNInstance<T, S>
where
    T: Service<(jsonrpc::Request, Option<S>)>,
    S: TryStream + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Bytes: From<S::Ok>,
{
    rpc: jsonrpc::Client<T, S>,
}

impl<T, S> RemoteDWNInstance<T, S>
where
    T: Service<
        (jsonrpc::Request, Option<S>),
        Response = jsonrpc::Response<(
            DWNResponse,
            BoxStream<'static, Result<Bytes, JSONRpcError>>,
        )>,
        Error = jsonrpc::JSONRpcError,
    >,
    S: TryStream + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Bytes: From<S::Ok>,
{
    pub fn new(transport: T) -> ClientResult<Self> {
        let rpc = jsonrpc::Client::new(transport);

        Ok(RemoteDWNInstance { rpc })
    }

    pub async fn process_message<D>(
        &mut self,
        tenant: &str,
        message: Message<D>,
        data: Option<S>,
    ) -> ClientResult<(DWNResponse, Option<impl Stream<Item = ClientResult<Bytes>>>)>
    where
        D: MessageDescriptor + DeserializeOwned + Serialize + Send + 'static,
    {
        let res = self
            .rpc
            .request(
                jsonrpc::dwn::PROCESS_MESSAGE,
                jsonrpc::dwn::ProcessMessageParams {
                    target: tenant.to_string(),
                    message,
                    encoded_data: None, // Data is always sent as a a stream
                },
                data,
            )
            .await?;

        let (m, d) = match res.result {
            jsonrpc::ResultError::Result(m) => Ok(m.reply),
            jsonrpc::ResultError::Error(e) => Err(JSONRpcError::from(e)),
        }?;

        let d = d.map(|d| match d {
            Ok(d) => Ok(d),
            Err(e) => Err(RemoteError::from(e)),
        });

        Ok((m, Some(d)))
    }
}

pub type RemoteHTTPDWNInstance<S> = RemoteDWNInstance<jsonrpc::HTTPTransport, S>;
pub fn new_remote_http_dwn<S>(url: String) -> ClientResult<RemoteHTTPDWNInstance<S>>
where
    S: TryStream + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Bytes: From<S::Ok>,
{
    let transport = jsonrpc::HTTPTransport::new(url)?;
    RemoteDWNInstance::new(transport)
}
