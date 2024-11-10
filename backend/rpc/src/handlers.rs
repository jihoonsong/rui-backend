use crate::AddMemberRequest;

#[async_trait::async_trait]
pub(crate) trait RpcApiAddMember {
    async fn add_member(&self, identity_commitment: AddMemberRequest);
}
