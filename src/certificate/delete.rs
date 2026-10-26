use std::sync::Mutex;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use sqlite::{Connection, State, State::Done};

use crate::auth::AuthDTO;

use super::CertificateDTO;

pub(crate) async fn delete_certificate(
    req: HttpRequest,
    certificate: web::Json<CertificateDTO>, //certificate.id
    db: web::Data<Mutex<Connection>>,
) -> impl Responder {
    let extensions = req.extensions();
    let auth = extensions
        .get::<AuthDTO>()
        .expect("jwt_auth middleware must run first");
    let db = db.lock().unwrap();

    let mut statement = db
        .prepare("DELETE FROM certificates WHERE id = ? AND user_id = ? RETURNING id;")
        .unwrap();

    statement
        .bind((1, certificate.id))
        .and_then(|_| statement.bind((2, auth.id)));

    match statement.next() {
        Ok(State::Row) => HttpResponse::Ok().body("Certificate Deleted"),
        Ok(Done) => HttpResponse::Ok().body("Certificate to delete not found"),
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}
