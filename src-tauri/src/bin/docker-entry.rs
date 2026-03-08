#![cfg(feature = "docker")]
//! Docker 入口程序 - 用于在 Docker 容器中运行 SeaLantern HTTP 服务器
//!
//! 此文件仅在启用了 `docker` feature 时编译（用于容器化运行时）。
//! 默认构建（无 `--features docker`）不会包含此二进制，从而避免在 `cargo run` 时产生二义性。

fn main() {
    // 调用库的 run 函数，它会自动检测 Docker 环境并启动 HTTP 服务器
    sea_lantern_lib::run();
}
