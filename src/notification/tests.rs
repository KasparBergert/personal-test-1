use super::*;

#[actix_web::test]
async fn returns_only_certificates_expired_before_now() {
    let db = sqlite::open(":memory:").unwrap();
    db.execute(crate::db::DB_INIT).unwrap();
    db.execute("INSERT INTO users (id, name, password, email) VALUES (1, 'test', 'password', 'test@example.com')")
        .unwrap();
    db.execute(
        "INSERT INTO certificates (id, user_id, title, description, expires_at) VALUES
            (1, 1, 'Expired', 'Past expiry', 1),
            (2, 1, 'Valid', 'Future expiry', 9223372036854775807)",
    )
    .unwrap();

    let db = web::Data::new(Mutex::new(db));
    let certificates = get_expiring_certificates(db).await.unwrap();

    assert_eq!(certificates.len(), 1);
    assert_eq!(certificates[0].0, "test@example.com");
    assert_eq!(certificates[0].1.id, 1);
}
