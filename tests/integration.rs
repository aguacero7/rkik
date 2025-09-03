use std::time::Duration;

#[tokio::test]
async fn test_query_invalid_host() {
    let err = rkik::query_one("no.such.domain.example", false, Duration::from_secs(1))
        .await
        .expect_err("expected error");
    assert!(matches!(err, rkik::RkikError::Dns(_)));
}