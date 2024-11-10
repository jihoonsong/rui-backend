use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use tracing::info;

use crate::{AddMemberRequest, RpcApiAddMember, RpcApiError, RpcApis, RpcError};

pub(crate) struct RpcApi {}

impl RpcApi {
    pub fn new() -> Self {
        Self {}
    }
}

#[rpc(server, namespace = "rui")]
pub(crate) trait RpcApi {
    #[method(name = "addMember")]
    async fn add_member(&self, request: AddMemberRequest) -> RpcResult<String>;
}

#[async_trait::async_trait]
impl<T> RpcApiServer for T
where
    T: RpcApis,
    jsonrpsee_types::ErrorObject<'static>: From<T::Error>,
{
    async fn add_member(&self, request: AddMemberRequest) -> RpcResult<String> {
        info!("add_member: {:?}", request);
        Ok(RpcApiAddMember::add_member(self, request).await?)
    }
}

impl RpcApiError for RpcApi {
    type Error = RpcError;
}

impl RpcApiAddMember for RpcApi {}
