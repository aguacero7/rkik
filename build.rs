fn main() {
    let mut features = vec![];
    if std::env::var("CARGO_FEATURE_JSON").is_ok() {
        features.push("json");
    }
    if std::env::var("CARGO_FEATURE_NTS").is_ok() {
        features.push("nts");
    }
    if std::env::var("CARGO_FEATURE_PTP").is_ok() {
        features.push("ptp");
    }
    if std::env::var("CARGO_FEATURE_SYNC").is_ok() {
        features.push("sync");
    }

    let features_str = if features.is_empty() {
        "none".to_string()
    } else {
        features.join(", ")
    };
    println!("cargo:rustc-env=RKIK_FEATURES={}", features_str);

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RKIK_TARGET={}", target);

    let rustc_ver = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().nth(1).map(|v| v.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=RKIK_RUSTC_VERSION={}", rustc_ver);
}
