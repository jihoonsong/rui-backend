use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use rui_backend_client::ClientHandlers;

use crate::{AddAnswerRequest, AddMemberRequest, RpcApiAddAnswer, RpcApiAddMember, RpcApis};

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

    #[method(name = "addAnswer")]
    async fn add_answer(&self, request: AddAnswerRequest) -> RpcResult<()>;
}

#[async_trait::async_trait]
impl<T> RpcApiServer for T
where
    T: RpcApis,
{
    async fn add_member(&self, request: AddMemberRequest) -> RpcResult<()> {
        Ok(RpcApiAddMember::add_member(self, request).await)
    }

    async fn add_answer(&self, request: AddAnswerRequest) -> RpcResult<()> {
        Ok(RpcApiAddAnswer::add_answer(self, request).await)
    }
}

#[async_trait::async_trait]
impl<H> RpcApiAddMember for RpcApi<H>
where
    H: ClientHandlers + Send + Sync + 'static,
{
    async fn add_member(&self, request: AddMemberRequest) {
        self.client_handlers
            .add_member(request.identity_commitment)
            .await;
    }
}

#[async_trait::async_trait]
impl<H> RpcApiAddAnswer for RpcApi<H>
where
    H: ClientHandlers + Send + Sync + 'static,
{
    async fn add_answer(&self, request: AddAnswerRequest) {
        self.client_handlers
            .add_answer(
                request.secret_bytes,
                request.message_bytes,
                request.scope_bytes,
                request.question_id,
                request.answer,
            )
            .await;
    }
}
