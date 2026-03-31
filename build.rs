fn main() {
    let mut features: Vec<String> = std::env::vars()
        .filter_map(|(key, _)| {
            const PREFIX: &str = "CARGO_FEATURE_";
            if let Some(stripped) = key.strip_prefix(PREFIX) {
                // Match cargo feature names by normalizing env var fragment:
                //   CARGO_FEATURE_JSON  -> "json"
                //   CARGO_FEATURE_SOME_FEATURE -> "some_feature"
                Some(stripped.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect();

    // Ensure deterministic ordering regardless of env var ordering
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

    let rustc_ver = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().nth(1).map(|v| v.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=RKIK_RUSTC_VERSION={}", rustc_ver);
}
