use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use std::net::SocketAddr;

use crate::{RpcApi, RpcApiServer, RpcError};

pub struct RpcServer {
    address: SocketAddr,
}

impl RpcServer {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub async fn build(&self) -> Result<ServerHandle, RpcError> {
        let rpc_api = RpcApi::new();

        let mut module = RpcModule::new(());
        module
            .merge(rpc_api.into_rpc())
            .map_err(|e| RpcError::Merge(String::from("rpc_api"), e))?;

        let server = Server::builder()
            .build(self.address)
            .await
            .map_err(|e| RpcError::Server(self.address, e))?;
        let server_handle = server.start(module);

        Ok(server_handle)
    }
}
