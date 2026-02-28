//! Docker 入口程序 - 用于在 Docker 容器中运行 SeaLantern HTTP 服务器

fn main() {
    // 调用库的 run 函数，它会自动检测 Docker 环境并启动 HTTP 服务器
    sea_lantern_lib::run();
}
