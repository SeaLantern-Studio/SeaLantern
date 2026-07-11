// License: GPL-3.0-only. Copyright (C) SeaLantern Studio.
//! Docker 入口程序 - 用于在 Docker 容器中运行 SeaLantern HTTP 服务器

fn main() {
    sea_lantern::run_headless_http();
}
