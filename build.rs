fn main() {
    let mut features: Vec<String> = std::env::vars()
        .filter_map(|(key, _)| {
            const PREFIX: &str = "CARGO_FEATURE_";
            key.strip_prefix(PREFIX)
                .map(|stripped| stripped.to_ascii_lowercase())
        })
        .collect();
    features.sort();
    features.dedup();

    let features_str = if features.is_empty() {
        "none".to_string()
    } else {
        features.join(", ")
    };
    println!("cargo:rustc-env=RKIK_FEATURES={}", features_str);

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RKIK_TARGET={}", target);

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let rustc_ver = std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().nth(1).map(|v| v.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=RKIK_RUSTC_VERSION={}", rustc_ver);
}
