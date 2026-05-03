use std::path::Path;
use std::time::{Duration, SystemTime};

use super::parser::{validate_starter_links_json, StarterLinksPayload};

///此处常量见 utils/constants.rs
use crate::utils::constants::STARTER_INSTALLER_LINKS_URL;

const STARTER_INSTALLER_LINKS_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const STARTER_INSTALLER_FETCH_RETRY_LIMIT: usize = 3;
const STARTER_INSTALLER_FETCH_RETRY_DELAY: Duration = Duration::from_secs(2);

pub(super) fn load_or_refresh_starter_links_json(links_file_path: &Path) -> Result<Vec<u8>, String> {
    if should_use_cached_links_file(links_file_path)? {
        return match read_and_validate_cached_links_file(links_file_path) {
            Ok(body) => Ok(body),
            Err(local_error) => {
                fetch_and_cache_starter_links_json(links_file_path).map_err(|refresh_error| {
                    format!(
                        "读取本地 Starter 下载信息失败: {}; 刷新远端 Starter 下载信息也失败: {}",
                        local_error, refresh_error
                    )
                })
            }
        };
    }

    match fetch_and_cache_starter_links_json(links_file_path) {
        Ok(body) => Ok(body),
        Err(refresh_error) => {
            if links_file_path.is_file() {
                return read_and_validate_cached_links_file(links_file_path).map_err(
                    |local_error| {
                        format!("{}；且本地 Starter 下载信息不可用: {}", refresh_error, local_error)
                    },
                );
            }
            Err(refresh_error)
        }
    }
}

fn read_and_validate_cached_links_file(links_file_path: &Path) -> Result<Vec<u8>, String> {
    let body = std::fs::read(links_file_path)
        .map_err(|e| format!("读取本地 Starter 下载信息失败: {}", e))?;
    validate_starter_links_json(&body)?;
    Ok(body)
}

fn fetch_and_cache_starter_links_json(links_file_path: &Path) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 Starter 请求客户端失败: {}", e))?;

    let mut attempt: usize = 0;

    let body = loop {
        attempt += 1;

        let response = match client.get(STARTER_INSTALLER_LINKS_URL).send() {
            Ok(response) => response,
            Err(e) => {
                if attempt >= STARTER_INSTALLER_FETCH_RETRY_LIMIT {
                    return Err(format!(
                        "请求 Starter 下载信息失败（第 {} 次尝试）: {}",
                        attempt, e
                    ));
                }
                std::thread::sleep(STARTER_INSTALLER_FETCH_RETRY_DELAY);
                continue;
            }
        };

        let status = response.status();
        if !status.is_success() {
            if attempt >= STARTER_INSTALLER_FETCH_RETRY_LIMIT {
                return Err(format!(
                    "Starter 下载接口返回异常状态（第 {} 次尝试）: {} ({})",
                    attempt, status, STARTER_INSTALLER_LINKS_URL
                ));
            }
            std::thread::sleep(STARTER_INSTALLER_FETCH_RETRY_DELAY);
            continue;
        }

        match response.bytes() {
            Ok(bytes) => break bytes.to_vec(),
            Err(e) => {
                if attempt >= STARTER_INSTALLER_FETCH_RETRY_LIMIT {
                    return Err(format!(
                        "读取 Starter 下载信息失败（第 {} 次尝试）: {}",
                        attempt, e
                    ));
                }
                std::thread::sleep(STARTER_INSTALLER_FETCH_RETRY_DELAY);
            }
        }
    };

    validate_starter_links_json(&body)
        .map_err(|e| format!("远端 Starter 下载信息校验失败: {}", e))?;

    std::fs::write(links_file_path, &body)
        .map_err(|e| format!("写入 Starter 下载信息失败: {}", e))?;

    Ok(body)
}

fn should_use_cached_links_file(links_file_path: &Path) -> Result<bool, String> {
    let metadata = match std::fs::metadata(links_file_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(false);
            }
            return Err(format!("读取 Starter 缓存文件元数据失败: {}", error));
        }
    };

    let modified_time = metadata
        .modified()
        .map_err(|e| format!("读取 Starter 缓存文件时间失败: {}", e))?;
    let age = SystemTime::now()
        .duration_since(modified_time)
        .unwrap_or(Duration::ZERO);

    Ok(age <= STARTER_INSTALLER_LINKS_CACHE_TTL)
}

#[allow(dead_code)] // 解析校验调用
fn _validate_payload(_payload: &StarterLinksPayload) {}
