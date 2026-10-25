use std::sync::Mutex;

use actix_web::{
    App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder, middleware::from_fn, web,
};
use serde::{Deserialize, Serialize};
use sqlite::{Connection, Row, State::Done, Statement};

mod auth;

use auth::{login, register};

use crate::auth::{AuthDTO, jwt_auth};

#[derive(Deserialize, Serialize)]
pub(crate) struct User {
    name: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct Certificate {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: String,
}

//make sure that get certificate can be used to edit, delete certificates only belonging to the user.

async fn create_certificate(
    req: HttpRequest,
    certificate: web::Json<Certificate>,
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

/** find user certificate by cert id and user_id and returns it */
async fn get_certificate(
    req: HttpRequest,
    certificate: web::Json<Certificate>,
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

async fn get_all_certificates(
    req: HttpRequest,
    certificate: web::Json<Certificate>,
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

async fn edit_certificate(db: web::Data<Mutex<Connection>>) -> impl Responder {
    //get auth
    let auth = req
        .extensions()
        .get::<AuthDTO>()
        .expect("jwt_auth middleware must run first");

    //get certificate id
    //get new certificate data

    //find certificate, change related data

    HttpResponse::Ok()
}

async fn delete_certificate(db: web::Data<Mutex<Connection>>) -> impl Responder {
    //get auth
    let auth = req
        .extensions()
        .get::<AuthDTO>()
        .expect("jwt_auth middleware must run first");

    //get certificate id

    //find certificate, delete it

    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //intialize the db
    let init = "
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS certificates (
    id INTEGER PRIMARY KEY,    
    user_id INTEGER NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
";

    let db = sqlite::open("./database.db").unwrap();
    db.execute(init).unwrap();

    let data = web::Data::new(Mutex::new(db));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .service(
                web::resource("/certificate")
                    .wrap(from_fn(jwt_auth))
                    .route(web::get().to(get_certificate))
                    .route(web::post().to(create_certificate))
                    .route(web::put().to(edit_certificate))
                    .route(web::delete().to(delete_certificate)),
            )
            .service(
                web::resource("/certificate/all")
                    .wrap(from_fn(jwt_auth))
                    .route(web::get().to(get_all_certificates)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
