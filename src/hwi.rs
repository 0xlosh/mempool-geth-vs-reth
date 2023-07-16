// source: https://github.com/gakonst/ethers-rs/blob/master/examples/providers/examples/custom.rs

use std::fmt::Debug;
use thiserror::Error;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use url::Url;
use ethers::prelude::*;

#[derive(Debug, Error)]
pub enum HWIError {
    #[error(transparent)]
    Ws(#[from] WsClientError),

    #[error(transparent)]
    Ipc(#[from] IpcError),

    #[error(transparent)]
    Http(#[from] HttpClientError),
}

impl RpcError for HWIError {
    fn as_error_response(&self) -> Option<&ethers::providers::JsonRpcError> {
        match self {
            HWIError::Ws(e) => e.as_error_response(),
            HWIError::Ipc(e) => e.as_error_response(),
            HWIError::Http(e) => e.as_error_response(),
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            HWIError::Ws(WsClientError::JsonError(e)) => Some(e),
            HWIError::Ipc(IpcError::JsonError(e)) => Some(e),
            HWIError::Http(HttpClientError::SerdeJson { err, .. }) => Some(err),
            _ => None,
        }
    }
}

impl From<HWIError> for ProviderError {
    fn from(value: HWIError) -> Self {
        Self::JsonRpcClientError(Box::new(value))
    }
}

#[derive(Debug, Clone)]
pub enum HWI {
    Ws(Ws),
    Ipc(Ipc),
    Http(Http),
}

impl HWI {
    pub async fn connect(s: &str) -> Result<Self, HWIError> {
        let this = match Url::parse(s) {
            Ok(url) => match url.scheme() {
                "http" | "https" => Self::Http(Http::new(url)),
                "ws" | "wss" => Self::Ws(Ws::connect(url).await?),
                _ => Self::Ipc(Ipc::connect(s).await?),
            },
            _ => Self::Ipc(Ipc::connect(s).await?),
        };
        Ok(this)
    }
}

#[async_trait]
impl JsonRpcClient for HWI {
    type Error = HWIError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let res = match self {
            Self::Ws(ws) => JsonRpcClient::request(ws, method, params).await?,
            Self::Ipc(ipc) => JsonRpcClient::request(ipc, method, params).await?,
            Self::Http(http) => JsonRpcClient::request(http, method, params).await?,
        };
        Ok(res)
    }
}

impl PubsubClient for HWI {
    type NotificationStream = <Ws as PubsubClient>::NotificationStream;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        let stream = match self {
            Self::Ws(ws) => PubsubClient::subscribe(ws, id)?,
            Self::Ipc(ipc) => PubsubClient::subscribe(ipc, id)?,
            _ => panic!("PubsubClient not available for HTTP"),
        };
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        match self {
            Self::Ws(ws) => PubsubClient::unsubscribe(ws, id)?,
            Self::Ipc(ipc) => PubsubClient::unsubscribe(ipc, id)?,
            _ => panic!("PubsubClient not available for HTTP"),
        };
        Ok(())
    }
}