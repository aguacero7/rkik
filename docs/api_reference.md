# API Reference

Public API exposed by `rkik` as a library. The API has no `clap`/`console` coupling and returns typed errors.

## Types

```rust
pub struct Target {
    pub name: String,
    pub ip: std::net::IpAddr,
    pub port: u16,
}

pub struct ProbeResult {
    pub target: Target,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    pub stratum: u8,
    pub ref_id: String,
    pub utc: chrono::DateTime<chrono::Utc>,
    pub local: chrono::DateTime<chrono::Local>,
    pub timestamp: i64,
}
```

> With the default `json` feature, these types derive `serde::Serialize` for easy serialization.

## Functions

```rust
pub async fn query_one(
    target: &str,
    ipv6_only: bool,
    timeout: std::time::Duration,
) -> Result<ProbeResult, RkikError>;

pub async fn compare_many(
    targets: &[String],
    ipv6_only: bool,
    timeout: std::time::Duration,
) -> Result<Vec<ProbeResult>, RkikError>;
```

## Example (library)

```rust
use rkik::{query_one, compare_many, ProbeResult, RkikError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), RkikError> {
    let r: ProbeResult = query_one("time.google.com", false, Duration::from_secs(3)).await?;
    println!("{} -> offset={}ms rtt={}ms", r.target.name, r.offset_ms, r.rtt_ms);

    let list = vec!["time.cloudflare.com".into(), "time.google.com".into()];
    let v = compare_many(&list, false, Duration::from_secs(3)).await?;
    for p in v { println!("{} {}", p.target.name, p.offset_ms); }
    Ok(())
}
```
