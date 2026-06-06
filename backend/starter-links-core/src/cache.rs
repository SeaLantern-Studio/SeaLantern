use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::{validate_starter_links_json, StarterLinksPayload};

const STARTER_INSTALLER_LINKS_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const STARTER_INSTALLER_FETCH_RETRY_LIMIT: usize = 3;
const STARTER_INSTALLER_FETCH_RETRY_DELAY: Duration = Duration::from_secs(2);

pub fn load_or_refresh_starter_links_json(
    links_file_path: &Path,
    remote_url: &str,
) -> Result<Vec<u8>, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("创建 Starter 异步运行时失败: {}", e))?
        .block_on(load_or_refresh_starter_links_json_inner(
            links_file_path,
            remote_url,
        ))
}

fn read_and_validate_cached_links_file(links_file_path: &Path) -> Result<Vec<u8>, String> {
    let body = std::fs::read(links_file_path)
        .map_err(|e| format!("读取本地 Starter 下载信息失败: {}", e))?;
    validate_starter_links_json(&body)?;
    Ok(body)
}

async fn load_or_refresh_starter_links_json_inner(
    links_file_path: &Path,
    remote_url: &str,
) -> Result<Vec<u8>, String> {
    if should_use_cached_links_file(links_file_path)? {
        return match read_and_validate_cached_links_file(links_file_path) {
            Ok(body) => Ok(body),
            Err(local_error) => fetch_and_cache_starter_links_json_async(links_file_path, remote_url)
                .await
                .map_err(|refresh_error| {
                    format!(
                        "读取本地 Starter 下载信息失败: {}; 刷新远端 Starter 下载信息也失败: {}",
                        local_error, refresh_error
                    )
                }),
        };
    }

    match fetch_and_cache_starter_links_json_async(links_file_path, remote_url).await {
        Ok(body) => Ok(body),
        Err(refresh_error) => recover_from_refresh_failure(links_file_path, refresh_error),
    }
}

fn recover_from_refresh_failure(
    links_file_path: &Path,
    refresh_error: String,
) -> Result<Vec<u8>, String> {
    if links_file_path.exists() {
        return read_and_validate_cached_links_file(links_file_path)
            .map_err(|local_error| {
                format!("{}；且本地 Starter 下载信息不可用: {}", refresh_error, local_error)
            });
    }

    Err(refresh_error)
}

async fn fetch_and_cache_starter_links_json_async(
    links_file_path: &Path,
    remote_url: &str,
) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 Starter 请求客户端失败: {}", e))?;

    let mut attempt: usize = 0;

    let body = loop {
        attempt += 1;

        let response = match client.get(remote_url).send().await {
            Ok(response) => response,
            Err(e) => {
                if attempt >= STARTER_INSTALLER_FETCH_RETRY_LIMIT {
                    return Err(format!(
                        "请求 Starter 下载信息失败（第 {} 次尝试）: {}",
                        attempt, e
                    ));
                }
                tokio::time::sleep(STARTER_INSTALLER_FETCH_RETRY_DELAY).await;
                continue;
            }
        };

        let status = response.status();
        if !status.is_success() {
            if attempt >= STARTER_INSTALLER_FETCH_RETRY_LIMIT {
                return Err(format!(
                    "Starter 下载接口返回异常状态（第 {} 次尝试）: {} ({})",
                    attempt, status, remote_url
                ));
            }
            tokio::time::sleep(STARTER_INSTALLER_FETCH_RETRY_DELAY).await;
            continue;
        }

        match response.bytes().await {
            Ok(bytes) => break bytes.to_vec(),
            Err(e) => {
                if attempt >= STARTER_INSTALLER_FETCH_RETRY_LIMIT {
                    return Err(format!(
                        "读取 Starter 下载信息失败（第 {} 次尝试）: {}",
                        attempt, e
                    ));
                }
                tokio::time::sleep(STARTER_INSTALLER_FETCH_RETRY_DELAY).await;
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
        .map_err(|e| format!("Starter 缓存文件修改时间异常: {}", e))?;

    Ok(age <= STARTER_INSTALLER_LINKS_CACHE_TTL)
}

#[allow(dead_code)]
fn _validate_payload(_payload: &StarterLinksPayload) {}

#[cfg(test)]
mod tests {
    use super::{recover_from_refresh_failure, should_use_cached_links_file};
    use std::time::{Duration, SystemTime};

    #[cfg(windows)]
    fn set_file_mtime(path: &std::path::Path, system_time: SystemTime) {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, INVALID_HANDLE_VALUE};
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, SetFileTime, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_BACKUP_SEMANTICS,
            FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
        };
        use windows::core::PCWSTR;

        let nanos_since_unix = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("mtime should be after unix epoch")
            .as_nanos();
        let windows_epoch_offset = 11644473600u128 * 10_000_000u128;
        let intervals = windows_epoch_offset + (nanos_since_unix / 100);
        let filetime = FILETIME {
            dwLowDateTime: intervals as u32,
            dwHighDateTime: (intervals >> 32) as u32,
        };

        let wide = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        unsafe {
            let handle = CreateFileW(
                PCWSTR(wide.as_ptr()),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_BACKUP_SEMANTICS,
                HANDLE::default(),
            )
            .expect("file handle should open");
            assert_ne!(handle, INVALID_HANDLE_VALUE, "file handle should be valid");
            let ok = SetFileTime(handle, None, None, Some(&filetime)).is_ok();
            let _ = CloseHandle(handle);
            assert!(ok, "mtime should be set");
        }
    }

    #[cfg(not(windows))]
    fn set_file_mtime(path: &std::path::Path, system_time: SystemTime) {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let duration = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("mtime should be after unix epoch");
        let times = [
            libc::timespec {
                tv_sec: duration.as_secs() as libc::time_t,
                tv_nsec: duration.subsec_nanos() as libc::c_long,
            },
            libc::timespec {
                tv_sec: duration.as_secs() as libc::time_t,
                tv_nsec: duration.subsec_nanos() as libc::c_long,
            },
        ];
        let c_path = CString::new(path.as_os_str().as_bytes()).expect("path should be c-safe");

        unsafe {
            let rc = libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0);
            assert_eq!(rc, 0, "mtime should be set");
        }
    }

    #[test]
    fn should_use_cached_links_file_rejects_future_modified_time() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let path = dir.path().join("jar_lfs_links.json");
        std::fs::write(&path, b"{}\n").expect("cache file should write");
        set_file_mtime(&path, SystemTime::now() + Duration::from_secs(3600));

        let error = should_use_cached_links_file(&path)
            .expect_err("future-dated cache files should not be treated as fresh");

        assert!(error.contains("Starter 缓存文件修改时间异常"));
    }

    #[test]
    fn recover_from_refresh_failure_reports_broken_local_path_when_path_exists() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let path = dir.path().join("jar_lfs_links.json");
        std::fs::create_dir(&path).expect("directory-backed cache path should exist");

        let error = recover_from_refresh_failure(&path, "远端刷新失败".to_string())
            .expect_err("existing invalid local cache path should be reported");

        assert!(error.contains("远端刷新失败"));
        assert!(error.contains("且本地 Starter 下载信息不可用"));
        assert!(error.contains("读取本地 Starter 下载信息失败"));
    }

    #[test]
    fn recover_from_refresh_failure_keeps_remote_error_when_local_path_missing() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let path = dir.path().join("jar_lfs_links.json");

        let error = recover_from_refresh_failure(&path, "远端刷新失败".to_string())
            .expect_err("missing local cache should preserve remote error");

        assert_eq!(error, "远端刷新失败");
    }
}
