use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use rui_backend_client::ClientHandlers;

use crate::{AddMemberRequest, RpcApiAddMember, RpcApis};

pub(crate) struct RpcApi<H>
where
    H: ClientHandlers + Send + Sync + 'static,
{
    client_handlers: H,
}

impl<H> RpcApi<H>
where
    H: ClientHandlers + Send + Sync + 'static,
{
    pub fn new(client_handlers: H) -> Self {
        Self { client_handlers }
    }
}

#[rpc(server, namespace = "rui")]
pub(crate) trait RpcApi {
    #[method(name = "addMember")]
    async fn add_member(&self, request: AddMemberRequest) -> RpcResult<()>;
}

#[async_trait::async_trait]
impl<T> RpcApiServer for T
where
    T: RpcApis,
{
    async fn add_member(&self, request: AddMemberRequest) -> RpcResult<()> {
        Ok(RpcApiAddMember::add_member(self, request).await)
    }
}

impl<H> RpcApiAddMember for RpcApi<H>
where
    H: ClientHandlers + Send + Sync + 'static,
{
    async fn add_member(&self, request: AddMemberRequest) {
        self.client_handlers.add_member(request.identity_commitment);
    }
}
