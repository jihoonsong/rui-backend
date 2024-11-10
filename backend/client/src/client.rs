use tracing::info;

use crate::ClientHandlers;

#[derive(Clone)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(self) {
        loop {}
    }
}

impl ClientHandlers for Client {
    fn add_member(&self, identity_commitment: String) {
        info!("add_member: {:?}", identity_commitment);
    }
}
