use std::sync::Mutex;

use crate::User;
use actix_web::{HttpResponse, Responder, web};
use sqlite::Connection;

pub async fn register(body: web::Json<User>, db: web::Data<Mutex<Connection>>) -> impl Responder {
    //get the name and password
    let body = body.into_inner();
    let db = db.lock().unwrap();

    println!("{:?}", body.name);
    println!("{:?}", body.password);

    let mut statement = db
        .prepare("INSERT INTO users (name, password) VALUES (?, ?)")
        .unwrap();

    let result = statement
        .bind((1, body.name.as_str()))
        .and_then(|_| statement.bind((2, body.password.as_str())))
        .and_then(|_| statement.next());

    match result {
        Ok(_res) => HttpResponse::Ok().body("User registered"),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
