use crate::{AddAnswerRequest, AddMemberRequest};

#[async_trait::async_trait]
pub(crate) trait RpcApiAddMember {
    async fn add_member(&self, request: AddMemberRequest);
}

#[async_trait::async_trait]
pub(crate) trait RpcApiAddAnswer {
    async fn add_answer(&self, request: AddAnswerRequest);
}
