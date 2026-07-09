fn main() {
    println!("cargo:rerun-if-env-changed=SEA_LANTERN_BUILD_VERSION");

    if let Ok(version) = std::env::var("SEA_LANTERN_BUILD_VERSION") {
        let trimmed = version.trim();
        if !trimmed.is_empty() {
            println!("cargo:rustc-env=SEA_LANTERN_BUILD_VERSION={trimmed}");
        }
    }

    tauri_build::build()
}
