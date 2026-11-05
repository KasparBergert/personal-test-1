mod auth;
mod certificate;
mod db;
mod notification;
#[cfg(test)]
mod test_utils;

use actix_web::{
    App, HttpServer,
    middleware::from_fn, web,
};
use auth::{jwt_auth, login, register};
use certificate::{
    create_certificate, delete_certificate, edit_certificate, get_all_certificates, get_certificate,
};
use chrono::{Local, Timelike};
use db::DB_INIT;
use notification::send_expiry_notifications;
use std::sync::Mutex;


//email of user and certificate

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = sqlite::open("./database.db").unwrap();
    db.execute(DB_INIT).unwrap();

    let data = web::Data::new(Mutex::new(db));
    let data2 = data.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            let time = Local::now();
            if time.hour() == 3 && time.minute() < 2 {
                if let Err(err) = send_expiry_notifications(data2.clone()).await {
                    eprintln!("{:?}", err);
                }
            }
        }
    });

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
