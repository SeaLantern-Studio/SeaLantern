use crate::services::server::log_pipeline as server_log_pipeline;

pub(super) fn run_preload_script(id: &str, server_path: &str) {
    #[cfg(target_os = "windows")]
    {
        let preload_script = std::path::Path::new(server_path).join("preload.bat");
        if preload_script.exists() {
            println!("发现预加载脚本: {:?}", preload_script);
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                "[preload] 开始执行预加载脚本...",
            );

            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/c", preload_script.to_str().unwrap_or("preload.bat")])
                .current_dir(server_path);

            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);

            match cmd.output() {
                Ok(result) => {
                    if result.status.success() {
                        println!("preload.bat 执行成功");
                        if !result.stdout.is_empty() {
                            let output = String::from_utf8_lossy(&result.stdout);
                            for line in output.lines() {
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    &format!("[preload] {}", line),
                                );
                            }
                        }
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            "[preload] 预加载脚本执行成功",
                        );
                    } else {
                        let error = String::from_utf8_lossy(&result.stderr);
                        println!("preload.bat 执行失败: {}", error);
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &format!("[preload] 执行失败: {}", error),
                        );
                    }
                }
                Err(e) => {
                    let error_msg = format!("执行 preload.bat 失败: {}", e);
                    println!("{}", error_msg);
                    let _ = server_log_pipeline::append_sealantern_log(
                        id,
                        &format!("[preload] {}", error_msg),
                    );
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let preload_script = std::path::Path::new(server_path).join("preload.sh");
        if preload_script.exists() {
            println!("发现预加载脚本: {:?}", preload_script);
            let _ = server_log_pipeline::append_sealantern_log(
                id,
                "[preload] 开始执行预加载脚本...",
            );

            match std::process::Command::new("sh")
                .arg(&preload_script)
                .current_dir(server_path)
                .output()
            {
                Ok(result) => {
                    if result.status.success() {
                        println!("preload.sh 执行成功");
                        if !result.stdout.is_empty() {
                            let output = String::from_utf8_lossy(&result.stdout);
                            for line in output.lines() {
                                let _ = server_log_pipeline::append_sealantern_log(
                                    id,
                                    &format!("[preload] {}", line),
                                );
                            }
                        }
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            "[preload] 预加载脚本执行成功",
                        );
                    } else {
                        let error = String::from_utf8_lossy(&result.stderr);
                        println!("preload.sh 执行失败: {}", error);
                        let _ = server_log_pipeline::append_sealantern_log(
                            id,
                            &format!("[preload] 执行失败: {}", error),
                        );
                    }
                }
                Err(e) => {
                    let error_msg = format!("执行 preload.sh 失败: {}", e);
                    println!("{}", error_msg);
                    let _ = server_log_pipeline::append_sealantern_log(
                        id,
                        &format!("[preload] {}", error_msg),
                    );
                }
            }
        }
    }
}
