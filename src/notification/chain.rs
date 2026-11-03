use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::web;
use sqlite::Connection;

use crate::certificate::Certificate;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

struct Email {
    from: String,
    to: String,
    title: String,
    body: String,
}

#[derive(Debug)]
pub enum ExpiryNotificationError {
    DatabaseLockPoisoned,
    Sqlite(sqlite::Error),
    SystemTime(std::time::SystemTimeError),
    TimestampOutOfRange(std::num::TryFromIntError),
    EmailRequest(reqwest::Error),
}

async fn get_expiring_certificates(
    db: web::Data<Mutex<Connection>>,
) -> Result<Vec<(String, Certificate)>, ExpiryNotificationError> {
    let db = db
        .lock()
        .map_err(|_| ExpiryNotificationError::DatabaseLockPoisoned)?;

    let mut statement = db
        .prepare(
            "SELECT users.email, certificates.*
             FROM certificates
             JOIN users ON users.id = certificates.user_id
             WHERE certificates.expires_at < ? AND users.email IS NOT NULL",
        )
        .map_err(ExpiryNotificationError::Sqlite)?;

    let now_sec = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(ExpiryNotificationError::SystemTime)?
            .as_secs(),
    )
    .map_err(ExpiryNotificationError::TimestampOutOfRange)?;

    statement
        .bind((1, now_sec))
        .map_err(ExpiryNotificationError::Sqlite)?;

    let mut result = Vec::new();

    loop {
        match statement.next().map_err(ExpiryNotificationError::Sqlite)? {
            sqlite::State::Row => {
                let email = statement
                    .read::<String, _>("email")
                    .map_err(ExpiryNotificationError::Sqlite)?;

                let certificate = Certificate {
                    id: statement
                        .read::<i64, _>("id")
                        .map_err(ExpiryNotificationError::Sqlite)?,
                    user_id: statement
                        .read::<i64, _>("user_id")
                        .map_err(ExpiryNotificationError::Sqlite)?,
                    title: statement
                        .read::<String, _>("title")
                        .map_err(ExpiryNotificationError::Sqlite)?,
                    description: statement
                        .read::<String, _>("description")
                        .map_err(ExpiryNotificationError::Sqlite)?,
                    expires_at: statement
                        .read::<i64, _>("expires_at")
                        .map_err(ExpiryNotificationError::Sqlite)?
                        .to_string(),
                };

                result.push((email, certificate));
            }
            sqlite::State::Done => break,
        }
    }

    Ok(result)
}

fn create_email(to: &str, title: &str, certificate: &Certificate) -> Email {
    let body = format!(
        "Hello,\n\nYour certificate \"{}\" expires on {}.\n\n",
        certificate.title, certificate.expires_at,
    );

    Email {
        from: "icertific@gmail.com".to_string(),
        to: to.to_owned(),
        title: title.to_owned(),
        body,
    }
}

async fn send_email(email: Email) -> Result<reqwest::Response, ExpiryNotificationError> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "From": { "Email": email.from },
        "To": [{ "Email": email.to }],
        "Subject": email.title,
        "Text": email.body,
    });

    client
        .post("http://localhost:8025/api/v1/send")
        .body(body.to_string())
        .send()
        .await
        .map_err(ExpiryNotificationError::EmailRequest)?
        .error_for_status()
        .map_err(ExpiryNotificationError::EmailRequest)
}

pub async fn send_expiry_notifications(
    db: web::Data<Mutex<Connection>>,
) -> Result<(), ExpiryNotificationError> {
    let certificate_list = get_expiring_certificates(db).await?;

    for (email_address, certificate) in certificate_list {
        let email = create_email(&email_address, "Certificate Expiry Reminder", &certificate);
        send_email(email).await?;
    }

    Ok(())
}
