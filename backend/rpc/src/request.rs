use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub identity_commitment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddAnswerRequest {
    pub question_id: String,
    pub answer: String,
}
