use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthDTO {
    pub id: i64,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct User {
    pub name: String,
    pub password: String,
    #[serde(default)]
    pub email: Option<String>,
}
