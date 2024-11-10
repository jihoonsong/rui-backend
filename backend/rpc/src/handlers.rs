use std::future::Future;

use crate::AddMemberRequest;

pub(crate) trait RpcApiAddMember {
    fn add_member(&self, request: AddMemberRequest) -> impl Future<Output = ()> + Send + Sync;
}
