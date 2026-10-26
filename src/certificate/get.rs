use std::sync::Mutex;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use sqlite::Connection;

use crate::auth::AuthDTO;

use super::{Certificate, CertificateDTO};

/** find user certificate by cert id and user_id and returns it */
pub(crate) async fn get_certificate(
    req: HttpRequest,
    certificate: web::Json<CertificateDTO>,
    db: web::Data<Mutex<Connection>>,
) -> impl Responder {
    let extensions = req.extensions();
    let auth = extensions
        .get::<AuthDTO>()
        .expect("jwt_auth middleware must run first");
    let db = db.lock().unwrap();

    //find certificates with his name on it
    let mut statement = db
        .prepare("SELECT * FROM certificates where user_id = ? AND id = ?;")
        .unwrap();

    statement
        .bind((1, auth.id))
        .and_then(|_| statement.bind((2, certificate.id)));

    match statement.next() {
        Ok(sqlite::State::Row) => {
            let certificate = (
                statement.read::<i64, _>("id"),
                statement.read::<String, _>("title"),
                statement.read::<String, _>("description"),
            );

            match certificate {
                (Ok(id), Ok(title), Ok(description)) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "id": id,
                        "title": title,
                        "description": description,
                    }))
                }
                _ => HttpResponse::InternalServerError().body("Could not read certificate"),
            }
        }
        Ok(sqlite::State::Done) => HttpResponse::NotFound().body("Certificate not found"),
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}

//make sure only certificates that are his can be retrieved
pub(crate) async fn get_all_certificates(
    req: HttpRequest,
    certificate: web::Json<CertificateDTO>,
    db: web::Data<Mutex<Connection>>,
) -> impl Responder {
    let extensions = req.extensions();
    let auth = extensions
        .get::<AuthDTO>()
        .expect("jwt_auth middleware must run first");
    let db = db.lock().unwrap();

    //find certificates with his name on it
    let mut statement = db
        .prepare("SELECT * FROM certificates where user_id = ?")
        .unwrap();

    statement
        .bind((1, auth.id))
        .and_then(|_| statement.bind((2, certificate.id)));

    let mut certificates: Vec<Certificate> = vec![];

    loop {
        match statement.next() {
            Ok(sqlite::State::Row) => {
                let certificate = (
                    statement.read::<i64, _>("id"),
                    statement.read::<String, _>("title"),
                    statement.read::<String, _>("description"),
                );

                match certificate {
                    (Ok(id), Ok(title), Ok(description)) => {
                        certificates.push(Certificate {
                            id,
                            title,
                            user_id: auth.id,
                            description,
                        });
                    }
                    _ => {
                        return HttpResponse::InternalServerError()
                            .body("Could not read certificate");
                    }
                }
            }
            Ok(sqlite::State::Done) => break,
            Err(_) => {
                return HttpResponse::InternalServerError().body("Database error");
            }
        }
    }

    HttpResponse::Ok().json(certificates)
}
