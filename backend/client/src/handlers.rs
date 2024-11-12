#[async_trait::async_trait]
pub trait ClientHandlers {
    async fn add_member(&self, identity_commitment: String);

    async fn add_answer(
        &self,
        secret_bytes: String,
        message_bytes: String,
        scope_bytes: String,
        question_id: String,
        answer: String,
    );
}
