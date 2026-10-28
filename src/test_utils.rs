#[macro_export]
macro_rules! init_test {
    () => {{
        let db = sqlite::open(":memory:").unwrap();
        db.execute($crate::db::DB_INIT).unwrap();
        let conn = actix_web::web::Data::new(std::sync::Mutex::new(db));

        actix_web::test::init_service(
            actix_web::App::new()
                .app_data(conn)
                .route(
                    "/register",
                    actix_web::web::post().to($crate::auth::register),
                )
                .route("/login", actix_web::web::post().to($crate::auth::login))
                .service(
                    actix_web::web::resource("/certificate")
                        .wrap(actix_web::middleware::from_fn($crate::auth::jwt_auth))
                        .route(actix_web::web::get().to($crate::certificate::get_certificate))
                        .route(actix_web::web::post().to($crate::certificate::create_certificate))
                        .route(actix_web::web::put().to($crate::certificate::edit_certificate))
                        .route(
                            actix_web::web::delete().to($crate::certificate::delete_certificate),
                        ),
                )
                .service(
                    actix_web::web::resource("/certificate/all")
                        .wrap(actix_web::middleware::from_fn($crate::auth::jwt_auth))
                        .route(actix_web::web::get().to($crate::certificate::get_all_certificates)),
                ),
        )
        .await
    }};
}
