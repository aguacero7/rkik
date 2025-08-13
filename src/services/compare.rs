use futures::future::join_all;
use std::time::Duration;

use crate::domain::ntp::ProbeResult;
use crate::error::RkikError;
use tracing::instrument;

use super::query::query_one;

/// Query many targets concurrently and return all successful [`ProbeResult`]s.
#[instrument(skip(timeout))]
pub async fn compare_many(
    targets: &[String],
    ipv6_only: bool,
    timeout: Duration,
) -> Result<Vec<ProbeResult>, RkikError> {
    let futures = targets
        .iter()
        .map(|t| query_one(t, ipv6_only, timeout))
        .collect::<Vec<_>>();
    let results = join_all(futures).await;
    let mut out = Vec::new();
    for res in results {
        out.push(res?);
    }
    Ok(out)
}
