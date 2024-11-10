use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use rui_backend_client::ClientHandlers;
use std::net::SocketAddr;

use crate::{RpcApi, RpcApiServer, RpcError};

pub struct RpcServer<H>
where
    H: ClientHandlers + Clone + Send + Sync + 'static,
{
    address: SocketAddr,
    client_handlers: H,
}

impl<H> RpcServer<H>
where
    H: ClientHandlers + Clone + Send + Sync + 'static,
{
    pub fn new(address: SocketAddr, client_handlers: H) -> Self {
        Self {
            address,
            client_handlers,
        }
    }

    pub async fn build(&self) -> Result<ServerHandle, RpcError> {
        let rpc_api = RpcApi::new(self.client_handlers.clone());

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
