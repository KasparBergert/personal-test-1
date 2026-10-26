use std::sync::Mutex;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use sqlite::Connection;

use crate::auth::AuthDTO;

use super::CertificateDTO;

pub(crate) async fn create_certificate(
    req: HttpRequest,
    certificate: web::Json<CertificateDTO>,
    db: web::Data<Mutex<Connection>>,
) -> impl Responder {
    let extensions = req.extensions();
    let auth = extensions
        .get::<AuthDTO>()
        .expect("jwt_auth middleware must run first");
    let db = db.lock().unwrap();

    let mut statement = db
        .prepare("INSERT INTO certificates (user_id, title, description) VALUES (?, ?, ?)")
        .unwrap();

    let result = statement
        .bind((1, auth.id))
        .and_then(|_| statement.bind((2, certificate.title.as_str())))
        .and_then(|_| statement.bind((3, certificate.description.as_str())))
        .and_then(|_| statement.next());

    let _result = match result {
        Ok(s) => s,
        Err(err) => {
            eprintln!(
                "SQLite error — code: {:?}, message: {:?}",
                err.code, err.message
            );
            return HttpResponse::InternalServerError().body("Could not save certificate");
        }
    };

    HttpResponse::Ok().body("Certificate created")
}
