use crate::{
    User,
    auth::jwt::{JWT, JWTType},
};
use actix_web::{
    HttpResponse, Responder,
    cookie::{Cookie, SameSite},
    web,
};
use serde::{Deserialize, Serialize};
use sqlite::Connection;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct AuthDTO {
    name: String,
}

pub async fn login(body: web::Json<User>, db: web::Data<Mutex<Connection>>) -> impl Responder {
    let body = body.into_inner();
    let db = db.lock().unwrap();

    let statement = db
        .prepare("SELECT password FROM users WHERE name = ?")
        .unwrap();

    let mut rows = match statement.into_iter().bind((1, body.name.as_str())) {
        Ok(rows) => rows,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };

    let row = match rows.next() {
        Some(Ok(row)) => row,
        Some(Err(error)) => return HttpResponse::InternalServerError().body(error.to_string()),
        None => return HttpResponse::BadRequest().body("User not found"),
    };

    let password = row.read::<&str, _>(0);
    if password != body.password {
        return HttpResponse::BadRequest().body("Could not log in");
    }

    let token = JWT::new(AuthDTO { name: body.name });

    let refresh = match token.to_jwt_string(JWTType::Refresh) {
        Ok(val) => val,
        Err(err) => {
            println!("{:?}", err.to_string());
            return HttpResponse::Forbidden().finish();
        }
    };

    let access = match token.to_jwt_string(JWTType::Access) {
        Ok(val) => val,
        Err(err) => {
            println!("{:?}", err.to_string());
            return HttpResponse::Forbidden().finish();
        }
    };

    let refresh_cookie = Cookie::build("refresh_token", refresh)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .finish();
    let access_cookie = Cookie::build("access_token", access)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .finish();

    HttpResponse::Ok()
        .cookie(refresh_cookie)
        .cookie(access_cookie)
        .body("Logged in")
}
