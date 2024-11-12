use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub identity_commitment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddAnswerRequest {
    pub secret_bytes: String,
    pub message_bytes: String,
    pub scope_bytes: String,
    pub question_id: String,
    pub answer: String,
}
