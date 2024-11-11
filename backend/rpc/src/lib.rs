mod api;
mod error;
mod handlers;
mod request;
mod server;

pub use error::RpcError;
pub use request::{AddAnswerRequest, AddMemberRequest};
pub use server::RpcServer;

pub(crate) use api::{RpcApi, RpcApiServer};
pub(crate) use handlers::{RpcApiAddAnswer, RpcApiAddMember};

pub(crate) trait RpcApis: RpcApiAddMember + RpcApiAddAnswer + Send + Sync + 'static {}

impl<T> RpcApis for T where T: RpcApiAddMember + RpcApiAddAnswer + Send + Sync + 'static {}
