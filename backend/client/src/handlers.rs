#[async_trait::async_trait]
pub trait ClientHandlers {
    async fn add_member(&self, identity_commitment: String);
}
