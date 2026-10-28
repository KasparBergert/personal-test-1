use crate::{
    User,
    auth::{ACCESS_TOKEN_COOKIE_NAME, REFRESH_TOKEN_COOKIE_NAME},
};
use actix_web::{http::header, test};

#[actix_web::test]
async fn registered_user_can_log_in_and_receive_tokens() {
    let app = crate::init_test!();
    let user = User {
        name: "test-user".into(),
        password: "test-password".into(),
    };

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/register")
            .set_json(&user)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/login")
            .set_json(&user)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let cookies: Vec<_> = response
        .headers()
        .get_all(header::SET_COOKIE)
        .map(|value| value.to_str().unwrap().to_owned())
        .collect();
    assert!(
        cookies
            .iter()
            .any(|cookie| cookie.starts_with(&format!("{ACCESS_TOKEN_COOKIE_NAME}=")))
    );
    assert!(
        cookies
            .iter()
            .any(|cookie| cookie.starts_with(&format!("{REFRESH_TOKEN_COOKIE_NAME}=")))
    );
}
