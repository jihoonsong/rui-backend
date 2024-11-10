use std::future::Future;

use tracing::info;

use crate::{AddMemberRequest, RpcApiError};

pub(crate) trait RpcApiAddMember {
    fn add_member(
        &self,
        request: AddMemberRequest,
    ) -> impl Future<Output = Result<String, Self::Error>> + Send
    where
        Self: RpcApiError,
    {
        async move {
            info!("add_member: {:?}", request);
            Ok(String::from("ok"))
        }
    }
}
