use std::sync::Mutex;

use actix_web::{App, HttpServer, web};
use serde::{Deserialize, Serialize};

mod auth;

use auth::{login, register};

#[derive(Deserialize, Serialize)]
pub(crate) struct User {
    name: String,
    password: String,
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
    db.execute(init).unwrap();

    let data = web::Data::new(Mutex::new(db));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
