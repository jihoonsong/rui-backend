use std::error::Error;
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("Failed to merge RPC endpoint {0}: {1}")]
    Merge(String, jsonrpsee::core::RegisterMethodError),

    #[error("Failed to build or start the server at {0}: {1}")]
    Server(SocketAddr, std::io::Error),
}

impl From<RpcError> for jsonrpsee_types::ErrorObject<'static> {
    fn from(error: RpcError) -> Self {
        match error {
            RpcError::Merge(endpoint, err) => jsonrpsee_types::ErrorObject::owned(
                jsonrpsee_types::error::INTERNAL_ERROR_CODE,
                "Failed to merge RPC endpoint",
                Some(format!("endpoint: {}, error: {}", endpoint, err)),
            ),
            RpcError::Server(socket, err) => jsonrpsee_types::ErrorObject::owned(
                jsonrpsee_types::error::INTERNAL_ERROR_CODE,
                "Failed to build or start the server",
                Some(format!("socket: {}, error: {}", socket, err)),
            ),
        }
    }
}

pub(crate) trait RpcApiError: Send + Sync {
    type Error: Into<jsonrpsee_types::ErrorObject<'static>> + Error + Send + Sync;
}
