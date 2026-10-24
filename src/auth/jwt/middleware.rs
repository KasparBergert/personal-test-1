use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

use crate::auth::{
    AuthDTO,
    jwt::{ACCESS_TOKEN_COOKIE_NAME, JWT},
};

pub async fn jwt_auth(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let access_jwt_raw = match req.cookie(ACCESS_TOKEN_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_owned(),
        None => {
            return Ok(req
                .into_response(HttpResponse::Unauthorized().finish())
                .map_into_right_body());
        }
    };

    let access_jwt =
        match JWT::<AuthDTO>::from_string_to_jwt(&access_jwt_raw, super::JWTType::Access) {
            Ok(jwt) => jwt,
            Err(err) => {
                return Ok(req
                    .into_response(HttpResponse::build(err.status_code()).finish())
                    .map_into_right_body());
            }
        };

    req.extensions_mut().insert(access_jwt.data);

    let response = next.call(req).await?;
    Ok(response.map_into_left_body())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, cookie::Cookie, middleware::from_fn, test};

    #[actix_web::test]
    async fn valid_access_cookie_passes_through_middleware() {
        let token = JWT::new(AuthDTO {
            name: "Kaspar".to_owned(),
        })
        .to_jwt_string(super::super::JWTType::Access)
        .unwrap();

        let app = test::init_service(App::new().wrap(from_fn(jwt_auth)).route(
            "/",
            actix_web::web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;

        let request = test::TestRequest::get()
            .uri("/")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, token))
            .to_request();

        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }
}
