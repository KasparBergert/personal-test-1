use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthDTO {
    pub id: i64,
}
