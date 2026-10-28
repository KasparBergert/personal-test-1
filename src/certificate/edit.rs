use std::sync::Mutex;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use sqlite::{Connection, State, State::Done};

use crate::auth::AuthDTO;

use super::CertificateDTO;

//test that any ids that are not created by the user cannot be edited
//equivalent of a put request
pub(crate) async fn edit_certificate(
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
        .prepare(
            "UPDATE certificates SET title = ?, description = ? WHERE id = ? AND user_id = ? RETURNING id;",
        )
        .unwrap();

    let _ = statement
        .bind((1, certificate.title.as_str()))
        .and_then(|_| statement.bind((2, certificate.description.as_str())))
        .and_then(|_| statement.bind((3, certificate.id)))
        .and_then(|_| statement.bind((4, auth.id)));

    match statement.next() {
        Ok(State::Row) => HttpResponse::Ok().body("Certificate edit completed"),
        Ok(Done) => HttpResponse::Ok().body("Certificate to edit not found"),
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}
