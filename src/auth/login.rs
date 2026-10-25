use crate::{
    User,
    auth::{
        AuthDTO,
        jwt::{ACCESS_TOKEN_COOKIE_NAME, JWT, JWTError, JWTType, REFRESH_TOKEN_COOKIE_NAME},
    },
};
use actix_web::{
    HttpResponse, Responder,
    cookie::{Cookie, SameSite},
    web,
};

use sqlite::Connection;
use std::sync::Mutex;

fn create_tokens<'a>(token: JWT<AuthDTO>) -> Result<(Cookie<'a>, Cookie<'a>), JWTError> {
    let refresh = token.to_jwt_string(JWTType::Refresh)?;
    let access = token.to_jwt_string(JWTType::Access)?;

    let refresh_cookie = Cookie::build(REFRESH_TOKEN_COOKIE_NAME, refresh)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .finish();
    let access_cookie = Cookie::build(ACCESS_TOKEN_COOKIE_NAME, access)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .finish();

    Ok((refresh_cookie, access_cookie))
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

    let jwt = JWT::new(AuthDTO { id: body.name });
    let (refresh_cookie, access_cookie) = match create_tokens(jwt) {
        Ok(cookies) => cookies,
        Err(err) => {
            println!("{}", err);
            return HttpResponse::new(err.status_code());
        }
    };

    HttpResponse::Ok()
        .cookie(refresh_cookie)
        .cookie(access_cookie)
        .body("Logged in")
}
