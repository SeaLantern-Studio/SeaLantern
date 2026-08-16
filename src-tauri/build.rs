use std::{env, path::PathBuf, process::Command};

fn build_macos_glass() {
    let target = env::var("TARGET").expect("Cargo must provide TARGET");
    let architecture = match target.as_str() {
        "aarch64-apple-darwin" => "arm64",
        "x86_64-apple-darwin" => "x86_64",
        _ => panic!("unsupported macOS target: {target}"),
    };
    let deployment_default = if architecture == "arm64" {
        "11.0"
    } else {
        "10.14"
    };
    let deployment =
        env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| deployment_default.to_owned());
    let swift_target = format!("{architecture}-apple-macosx{deployment}");
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must provide OUT_DIR"));
    let library_path = output_dir.join("libsealantern_macos.a");
    let source_path = PathBuf::from("macos/LiquidGlass.swift");

    let status = Command::new("xcrun")
        .args(["swiftc", "-parse-as-library", "-emit-library", "-static"])
        .args(["-target", &swift_target, "-module-name", "SeaLanternMacOS"])
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .status()
        .expect("failed to start the Swift compiler through xcrun");
    assert!(status.success(), "failed to compile macOS Liquid Glass");

    let swiftc_output = Command::new("xcrun")
        .args(["--find", "swiftc"])
        .output()
        .expect("failed to locate the Swift compiler");
    assert!(swiftc_output.status.success(), "failed to locate swiftc");
    let swiftc_path = PathBuf::from(
        String::from_utf8(swiftc_output.stdout)
            .expect("swiftc path must be UTF-8")
            .trim(),
    );
    let swift_library_dir = swiftc_path
        .parent()
        .and_then(|path| path.parent())
        .expect("swiftc must be inside the toolchain usr/bin directory")
        .join("lib/swift/macosx");

    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    println!("cargo:rustc-link-search=native={}", output_dir.display());
    println!("cargo:rustc-link-search=native={}", swift_library_dir.display());
    println!("cargo:rustc-link-lib=static=sealantern_macos");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
}

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        build_macos_glass();
    }
    tauri_build::build()
}
