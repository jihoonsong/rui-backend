mod api;
mod error;
mod handlers;
mod request;
mod server;

pub use error::RpcError;
pub use request::AddMemberRequest;
pub use server::RpcServer;

pub(crate) use api::{RpcApi, RpcApiServer};
pub(crate) use handlers::RpcApiAddMember;

pub(crate) trait RpcApis: RpcApiAddMember + Send + Sync + 'static {}

impl<T> RpcApis for T where T: RpcApiAddMember + Send + Sync + 'static {}
