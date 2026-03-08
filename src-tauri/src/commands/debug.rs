//! debug.rs
//! 仅在 debug 构建（`pnpm run tauri dev`）下编译的调试专用命令。
//!
//! 所有命令均使用 `#[cfg(debug_assertions)]` 守卫，
//! 在执行 `pnpm run tauri build` 生成发布包时不会被编译，
//! 因此不会向最终用户暴露任何调试接口。

/// 主动触发一个 panic，用于测试 `init_panic_hook` 注册的崩溃日志功能是否正常工作。
///
/// # 使用方式
/// 启动开发服务器后，在浏览器开发者工具的控制台中执行：
///
/// ```js
/// await window.__invoke("debug_panic")
/// ```
///
/// 触发后程序会立即崩溃，panic hook 会：
/// 1. 在项目根目录的 `panic-log/` 文件夹下生成以时间戳命名的 `.log` 文件
/// 2. 将日志内容输出到终端 stderr
/// 3. 以退出码 0xFFFF 终止进程
#[cfg(debug_assertions)]
#[tauri::command]
pub fn debug_panic() {
    // 主动 panic，消息会出现在日志的 Message 字段中
    panic!("手动触发的测试 panic：用于验证 init_panic_hook 崩溃日志功能是否正常工作");
}
