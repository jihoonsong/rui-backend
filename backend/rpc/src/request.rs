use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub identity_commitment_bytes: Vec<u8>,
}
