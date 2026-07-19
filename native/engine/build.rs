fn main() {
    let version = std::env::var("NSDOWNLOAD_APP_VERSION")
        .unwrap_or_else(|_| "1.0".to_string());
    println!("cargo:rustc-env=NSDOWNLOAD_APP_VERSION={version}");
    println!("cargo:rerun-if-env-changed=NSDOWNLOAD_APP_VERSION");
}
