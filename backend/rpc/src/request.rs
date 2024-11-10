use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub identity_commitment: String,
}
