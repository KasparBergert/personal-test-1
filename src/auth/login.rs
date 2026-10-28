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

use sqlite::{Connection, State};
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

    let mut statement = db
        .prepare("SELECT password FROM users WHERE name = ?;")
        .unwrap();

    let _ = statement.bind((1, body.name.as_str()));

    let password = match statement.next() {
        Ok(State::Row) => match statement.read::<String, _>(0) {
            Ok(password) => password,
            Err(_) => return HttpResponse::InternalServerError().body("Database error"),
        },
        Ok(State::Done) => return HttpResponse::BadRequest().body("Could not log in"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    if password != body.password {
        return HttpResponse::BadRequest().body("Could not log in");
    }

    //get the id of the user
    let mut statement = db.prepare("SELECT id FROM users WHERE name = ?;").unwrap();
    let _ = statement.bind((1, body.name.as_str()));

    let id = match statement.next() {
        Ok(State::Row) => match statement.read::<i64, _>(0) {
            Ok(id) => id,
            Err(_) => return HttpResponse::InternalServerError().body("Database error"),
        },
        Ok(State::Done) => return HttpResponse::BadRequest().body("Could not log in"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let jwt = JWT::new(AuthDTO { id });
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
