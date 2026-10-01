use std::{fs, sync::Mutex};

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use sqlite::Connection;

async fn register(db: web::Data<Connection>) -> impl Responder {
    // Use the connection here
    "register"
}

async fn login(db: web::Data<Connection>) -> impl Responder {
    // Use the connection here
    "login"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //intialize the db
    let init = "
    CREATE TABLE IF NOT EXISTS users (
        name TEXT NOT NULL UNIQUE,
        password TEXT NOT NULL
    );
    ";

    let db = sqlite::open("./database.db").unwrap();

    let data = web::Data::new(Mutex::new(db));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/get", web::post().to(register))
            .route("/get", web::post().to(login))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
