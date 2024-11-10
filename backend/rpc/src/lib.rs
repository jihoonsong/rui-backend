mod api;
mod error;
mod handlers;
mod request;
mod server;

pub use error::RpcError;
pub use request::AddMemberRequest;
pub use server::RpcServer;

pub(crate) use api::{RpcApi, RpcApiServer};
pub(crate) use error::RpcApiError;
pub(crate) use handlers::RpcApiAddMember;

pub(crate) trait RpcApis: RpcApiError + RpcApiAddMember + 'static {}

impl<T> RpcApis for T where T: RpcApiError + RpcApiAddMember + 'static {}
