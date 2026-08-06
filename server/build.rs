//! 构建脚本：为 Windows 可执行文件嵌入图标与版本信息。
//!
//! 仅 Windows 编译时生效；其他平台（Linux/macOS）为无操作。
//! 版本号从 `CARGO_PKG_VERSION` 动态读取，避免与 Cargo.toml 双份维护。

fn main() {
    #[cfg(target_os = "windows")]
    embed_windows_resources();
}

#[cfg(target_os = "windows")]
fn embed_windows_resources() {
    let mut resource = winres::WindowsResource::new();

    // 图标：复用 src-tauri 同款素材，置于 server crate 根。
    resource.set_icon("icons/server.ico");

    // 文件版本：从 Cargo.toml 的 version 解析为四段数字。
    let version = parse_version(env!("CARGO_PKG_VERSION"));
    resource.set("FileVersion", &version);
    resource.set("ProductVersion", &version);

    resource.set("FileDescription", "Sea Lantern Server");
    resource.set("ProductName", "Sea Lantern");
    resource.set("CompanyName", "DragonHTDev");
    resource.set("LegalCopyright", "Copyright (c) 2026 SeaLantern Studio");

    resource
        .compile()
        .expect("failed to embed Windows resources");
}

/// 将 `x.y.z` 形式的版本号规范化为 Windows 的 `x,y,z,0` 四段格式。
#[cfg(target_os = "windows")]
fn parse_version(version: &str) -> String {
    let mut parts: Vec<String> = version
        .split('.')
        .map(|part| part.chars().filter(|c| c.is_ascii_digit()).collect())
        .filter(|part: &String| !part.is_empty())
        .collect();
    while parts.len() < 4 {
        parts.push("0".to_string());
    }
    parts.truncate(4);
    parts.join(",")
}
