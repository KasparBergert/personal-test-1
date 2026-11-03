use serde::{Deserialize, Serialize};

// Shared with the internet.
#[derive(Deserialize, Serialize)]
pub(crate) struct CertificateDTO {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub expires_at: String,
}

// Kept private to the application.
#[derive(Deserialize, Serialize)]
pub(crate) struct Certificate {
    pub id: i64,
    pub title: String,
    pub user_id: i64,
    pub description: String,
    pub expires_at: String,
}
