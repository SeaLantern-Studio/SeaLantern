use super::state::{ApiResponse, AppState, UploadFailure};
use crate::utils::logger::{capture_eprintln, capture_println};
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use std::path::{Path as FsPath, PathBuf};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use uuid::Uuid;

/// 处理文件上传请求
pub(super) async fn handle_file_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    match save_uploaded_files(&state, &mut multipart).await {
        Ok(uploaded_files) => {
            capture_println(format!(
                "[Upload] Successfully uploaded {} file(s)",
                uploaded_files.len()
            ));
            (
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "files": uploaded_files,
                    "count": uploaded_files.len()
                }))),
            )
                .into_response()
        }
        Err(error) => {
            capture_eprintln(format!("[Upload] {}", error.message));
            (error.status, Json(ApiResponse::error(error.message))).into_response()
        }
    }
}

async fn save_uploaded_files(
    state: &AppState,
    multipart: &mut Multipart,
) -> Result<Vec<Value>, UploadFailure> {
    let upload_root = ensure_upload_root(&state.config.upload_dir).await?;
    let mut uploaded_files = Vec::new();
    let mut committed_paths = Vec::new();
    let mut file_count = 0usize;

    loop {
        let next_field = multipart.next_field().await.map_err(|error| {
            UploadFailure::new(
                error.status(),
                format!("Failed to parse multipart payload: {}", error.body_text()),
            )
        })?;

        let Some(mut field) = next_field else {
            break;
        };

        file_count += 1;
        if file_count > state.config.max_upload_files {
            cleanup_saved_files(&committed_paths).await;
            return Err(UploadFailure::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "Too many files in a single upload request (max {})",
                    state.config.max_upload_files
                ),
            ));
        }

        let original_name = field
            .file_name()
            .ok_or_else(|| {
                UploadFailure::new(
                    StatusCode::BAD_REQUEST,
                    "Upload field is missing a filename".to_string(),
                )
            })?
            .to_string();
        let safe_name = sanitize_upload_basename(&original_name).map_err(|message| {
            UploadFailure::new(
                StatusCode::BAD_REQUEST,
                format!("Invalid upload filename '{}': {}", original_name, message),
            )
        })?;

        let (mut file, save_path) = open_unique_upload_file(&upload_root, &safe_name).await?;
        let mut size = 0usize;

        let write_result = async {
            while let Some(chunk) = field.chunk().await.map_err(|error| {
                UploadFailure::new(
                    error.status(),
                    format!(
                        "Failed to read uploaded file '{}': {}",
                        original_name,
                        error.body_text()
                    ),
                )
            })? {
                size = size.saturating_add(chunk.len());
                if size > state.config.max_upload_file_bytes {
                    return Err(UploadFailure::new(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "Uploaded file '{}' exceeds the single-file size limit of {} bytes",
                            original_name, state.config.max_upload_file_bytes
                        ),
                    ));
                }

                file.write_all(&chunk).await.map_err(|error| {
                    UploadFailure::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to save uploaded file '{}': {}", original_name, error),
                    )
                })?;
            }

            file.flush().await.map_err(|error| {
                UploadFailure::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to finalize uploaded file '{}': {}", original_name, error),
                )
            })?;

            Ok::<(), UploadFailure>(())
        }
        .await;

        if let Err(error) = write_result {
            cleanup_saved_files(&committed_paths).await;
            let _ = fs::remove_file(&save_path).await;
            return Err(error);
        }

        capture_println(format!(
            "[Upload] File '{}' saved to '{}'",
            original_name,
            save_path.display()
        ));

        committed_paths.push(save_path.clone());
        uploaded_files.push(serde_json::json!({
            "original_name": original_name,
            "saved_name": save_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
            "saved_path": save_path.to_string_lossy(),
            "size": size
        }));
    }

    if uploaded_files.is_empty() {
        return Err(UploadFailure::new(StatusCode::BAD_REQUEST, "No files uploaded".to_string()));
    }

    Ok(uploaded_files)
}

async fn ensure_upload_root(upload_dir: &FsPath) -> Result<PathBuf, UploadFailure> {
    fs::create_dir_all(upload_dir).await.map_err(|error| {
        UploadFailure::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to prepare upload directory '{}': {}", upload_dir.display(), error),
        )
    })?;

    fs::canonicalize(upload_dir).await.map_err(|error| {
        UploadFailure::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to resolve upload directory '{}': {}", upload_dir.display(), error),
        )
    })
}

async fn open_unique_upload_file(
    upload_root: &FsPath,
    safe_name: &str,
) -> Result<(tokio::fs::File, PathBuf), UploadFailure> {
    for _ in 0..8 {
        let save_name = build_unique_saved_name(safe_name);
        let save_path = build_upload_target_path(upload_root, &save_name)
            .map_err(|message| UploadFailure::new(StatusCode::BAD_REQUEST, message))?;

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&save_path)
            .await
        {
            Ok(file) => return Ok((file, save_path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(UploadFailure::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to open upload target '{}': {}", save_path.display(), error),
                ));
            }
        }
    }

    Err(UploadFailure::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to allocate a unique upload filename after several attempts".to_string(),
    ))
}

pub(super) fn build_upload_target_path(upload_root: &FsPath, raw_name: &str) -> Result<PathBuf, String> {
    let safe_name = sanitize_upload_basename(raw_name)?;
    let candidate = upload_root.join(&safe_name);

    if candidate.starts_with(upload_root) {
        Ok(candidate)
    } else {
        Err(format!(
            "Resolved upload path '{}' escapes the upload directory '{}'",
            candidate.display(),
            upload_root.display()
        ))
    }
}

pub(super) fn sanitize_upload_basename(raw_name: &str) -> Result<String, String> {
    let normalized = raw_name.replace('\\', "/");
    let basename = normalized
        .rsplit('/')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .trim_end_matches([' ', '.']);

    if basename.is_empty() || basename == "." || basename == ".." {
        return Err("filename must contain a non-empty basename".to_string());
    }

    if basename.chars().any(char::is_control) {
        return Err("filename contains control characters".to_string());
    }

    if basename.contains('/') || basename.contains('\\') {
        return Err("filename contains path separators after normalization".to_string());
    }

    let reserved = basename
        .split('.')
        .next()
        .unwrap_or(basename)
        .trim()
        .to_ascii_uppercase();
    if is_reserved_platform_name(&reserved) {
        return Err("filename uses a platform-reserved basename".to_string());
    }

    Ok(basename.to_string())
}

fn is_reserved_platform_name(name: &str) -> bool {
    matches!(
        name,
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

pub(super) fn build_unique_saved_name(safe_name: &str) -> String {
    let path = FsPath::new(safe_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("upload");
    let extension = path.extension().and_then(|value| value.to_str());
    let suffix = Uuid::new_v4().simple().to_string();

    match extension {
        Some(extension) if !extension.is_empty() => format!("{}-{}.{}", stem, suffix, extension),
        _ => format!("{}-{}", stem, suffix),
    }
}

async fn cleanup_saved_files(paths: &[PathBuf]) {
    for path in paths {
        if let Err(error) = fs::remove_file(path).await {
            capture_eprintln(format!(
                "[Upload] Failed to clean up partial upload '{}': {}",
                path.display(),
                error
            ));
        }
    }
}
