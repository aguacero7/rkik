use std::time::Duration;

#[tokio::test]
async fn test_query_invalid_host() {
    let err = rkik::query_one(
        "no.such.domain.example",
        false,                        // ipv6
        Duration::from_secs(1),       // timeout
        false,                        // use_nts
        4460,                         // nts_port
    )
    .await
    .expect_err("expected error");
    assert!(matches!(err, rkik::RkikError::Dns(_)));
}
