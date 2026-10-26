use super::*;
use crate::{
    DB_INIT,
    auth::{ACCESS_TOKEN_COOKIE_NAME, AuthDTO, JWT, JWTType, jwt_auth},
};
use actix_web::{App, cookie::Cookie, http::StatusCode, test, web};
use std::sync::Mutex;

macro_rules! init_test {
    () => {{
        let db = sqlite::open(":memory:").unwrap();
        db.execute(DB_INIT).unwrap();
        let conn = web::Data::new(Mutex::new(db));

        test::init_service(
            App::new()
                .app_data(conn.clone())
                .service(
                    web::resource("/certificate")
                        .wrap(actix_web::middleware::from_fn(jwt_auth))
                        .route(web::get().to(get_certificate))
                        .route(web::post().to(create_certificate))
                        .route(web::put().to(edit_certificate))
                        .route(web::delete().to(delete_certificate)),
                )
                .service(
                    web::resource("/certificate/all")
                        .wrap(actix_web::middleware::from_fn(jwt_auth))
                        .route(web::get().to(get_all_certificates)),
                ),
        )
        .await
    }};
}

#[actix_web::test]
async fn owner_can_get_their_certificate_but_not_another_users_certificate() {
    let app = init_test!();

    let owner_token = JWT::new(AuthDTO { id: 1 })
        .to_jwt_string(JWTType::Access)
        .unwrap();
    let other_token = JWT::new(AuthDTO { id: 2 })
        .to_jwt_string(JWTType::Access)
        .unwrap();

    // Assumes users 1 and 2 exist if your certificates table has a foreign key.
    let owner_certificate = CertificateDTO {
        id: 0,
        title: "Rust certificate".into(),
        description: "Owned by owner".into(),
    };
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner_token.clone()))
            .set_json(&owner_certificate)
            .to_request(),
    )
    .await;

    let other_certificate = CertificateDTO {
        id: 0,
        title: "Other certificate".into(),
        description: "Owned by other user".into(),
    };
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, other_token))
            .set_json(&other_certificate)
            .to_request(),
    )
    .await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner_token.clone()))
            .set_json(&CertificateDTO {
                id: 1,
                ..owner_certificate
            })
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner_token))
            .set_json(&CertificateDTO {
                id: 2,
                ..other_certificate
            })
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn get_all_certificates_returns_only_certificates_owned_by_authenticated_user() {
    let app = init_test!();

    let owner = JWT::new(AuthDTO { id: 1 })
        .to_jwt_string(JWTType::Access)
        .unwrap();
    let other = JWT::new(AuthDTO { id: 2 })
        .to_jwt_string(JWTType::Access)
        .unwrap();

    for (token, title) in [
        (owner.clone(), "First owned"),
        (owner.clone(), "Second owned"),
        (other, "Not owned"),
    ] {
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/certificate")
                .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, token))
                .set_json(CertificateDTO {
                    id: 0,
                    title: title.into(),
                    description: String::new(),
                })
                .to_request(),
        )
        .await;
    }

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/certificate/all")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner))
            .set_json(CertificateDTO {
                id: 0,
                title: String::new(),
                description: String::new(),
            })
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let certificates: Vec<Certificate> = test::read_body_json(response).await;
    assert_eq!(certificates.len(), 2);
    assert!(
        certificates
            .iter()
            .all(|certificate| certificate.user_id == 1)
    );
}

#[actix_web::test]
async fn owner_can_edit_their_certificate_but_another_user_cannot_change_it() {
    let app = init_test!();

    let owner = JWT::new(AuthDTO { id: 1 })
        .to_jwt_string(JWTType::Access)
        .unwrap();
    let other = JWT::new(AuthDTO { id: 2 })
        .to_jwt_string(JWTType::Access)
        .unwrap();

    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner.clone()))
            .set_json(CertificateDTO {
                id: 0,
                title: "Original".into(),
                description: "Original description".into(),
            })
            .to_request(),
    )
    .await;

    let edit = |token: String, title: &str, description: &str| {
        test::TestRequest::put()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, token))
            .set_json(CertificateDTO {
                id: 1,
                title: title.into(),
                description: description.into(),
            })
            .to_request()
    };

    let response =
        test::call_service(&app, edit(owner.clone(), "Updated", "Updated description")).await;
    assert_eq!(response.status(), StatusCode::OK);

    test::call_service(&app, edit(other, "Stolen", "Stolen description")).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner))
            .set_json(CertificateDTO {
                id: 1,
                title: String::new(),
                description: String::new(),
            })
            .to_request(),
    )
    .await;

    let certificate: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(certificate["title"], "Updated");
    assert_eq!(certificate["description"], "Updated description");
}

#[actix_web::test]
async fn owner_can_delete_their_certificate_but_another_user_cannot_delete_it() {
    let app = init_test!();

    let owner = JWT::new(AuthDTO { id: 1 })
        .to_jwt_string(JWTType::Access)
        .unwrap();
    let other = JWT::new(AuthDTO { id: 2 })
        .to_jwt_string(JWTType::Access)
        .unwrap();

    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner.clone()))
            .set_json(CertificateDTO {
                id: 0,
                title: "Owner certificate".into(),
                description: "Protected".into(),
            })
            .to_request(),
    )
    .await;

    let delete = |token: String| {
        test::TestRequest::delete()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, token))
            .set_json(CertificateDTO {
                id: 1,
                title: String::new(),
                description: String::new(),
            })
            .to_request()
    };

    test::call_service(&app, delete(other)).await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner.clone()))
            .set_json(CertificateDTO {
                id: 1,
                title: String::new(),
                description: String::new(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(
        test::call_service(&app, delete(owner.clone()))
            .await
            .status(),
        StatusCode::OK
    );

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/certificate")
            .cookie(Cookie::new(ACCESS_TOKEN_COOKIE_NAME, owner))
            .set_json(CertificateDTO {
                id: 1,
                title: String::new(),
                description: String::new(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
