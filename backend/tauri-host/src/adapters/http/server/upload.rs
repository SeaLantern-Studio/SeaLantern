use super::state::{ApiErrorDetail, ApiResponse, AppState, UploadFailure};
use crate::utils::logger::{log_error_ctx, log_info_ctx, log_warn_ctx};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use std::path::{Path as FsPath, PathBuf};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use uuid::Uuid;

const UPLOAD_REFERENCE_PREFIX: &str = "upload://";

fn log_upload_info(function: &str, message: &str) {
    log_info_ctx("http.server.upload", function, message);
}

fn log_upload_warn(function: &str, message: &str) {
    log_warn_ctx("http.server.upload", function, message);
}

fn log_upload_error(function: &str, message: &str) {
    log_error_ctx("http.server.upload", function, message);
}

fn upload_error_kind_for_status(status: StatusCode) -> &'static str {
    if status.is_server_error() {
        "runtime"
    } else {
        "invalid_request"
    }
}

/// 处理文件上传请求
pub(super) async fn handle_file_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    match save_uploaded_files(&state, &mut multipart).await {
        Ok(uploaded_files) => {
            log_upload_info(
                "handle_file_upload",
                &format!("successfully uploaded {} file(s)", uploaded_files.len()),
            );
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
            if error.status.is_server_error() {
                log_upload_error("handle_file_upload", &error.message);
            } else {
                log_upload_warn("handle_file_upload", &error.message);
            }
            (
                error.status,
                Json(ApiResponse::error_with_detail(
                    error.message.clone(),
                    ApiErrorDetail {
                        code: "common.message_unknown_error".to_string(),
                        message: error.message,
                        args: None,
                        error_kind: Some(upload_error_kind_for_status(error.status).to_string()),
                    },
                )),
            )
                .into_response()
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

        log_upload_info(
            "save_uploaded_files",
            &format!("file '{}' saved to '{}'", original_name, save_path.display()),
        );

        committed_paths.push(save_path.clone());
        let saved_name = save_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        uploaded_files.push(serde_json::json!({
            "original_name": original_name,
            "saved_name": saved_name,
            "saved_path": build_upload_reference(&saved_name),
            "size": size
        }));
    }

    if uploaded_files.is_empty() {
        return Err(UploadFailure::new(StatusCode::BAD_REQUEST, "No files uploaded".to_string()));
    }

    Ok(uploaded_files)
}

pub(super) async fn resolve_uploaded_value_references(
    upload_dir: &FsPath,
    params: Value,
) -> Result<Value, UploadFailure> {
    if !value_contains_upload_reference(&params) {
        return Ok(params);
    }

    let upload_root = ensure_upload_root(upload_dir).await?;
    rewrite_upload_references_in_value(params, &upload_root)
        .map_err(|message| UploadFailure::new(StatusCode::BAD_REQUEST, message))
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

pub(super) fn build_upload_target_path(
    upload_root: &FsPath,
    raw_name: &str,
) -> Result<PathBuf, String> {
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

pub(super) fn build_upload_reference(saved_name: &str) -> String {
    format!("{}{}", UPLOAD_REFERENCE_PREFIX, saved_name)
}

pub(super) fn resolve_upload_reference(
    upload_root: &FsPath,
    raw_value: &str,
) -> Result<Option<PathBuf>, String> {
    let Some(saved_name) = raw_value.strip_prefix(UPLOAD_REFERENCE_PREFIX) else {
        return Ok(None);
    };

    let safe_name = sanitize_upload_basename(saved_name)?;
    let save_path = build_upload_target_path(upload_root, &safe_name)?;
    if !save_path.is_file() {
        return Err(format!("Uploaded file reference not found: {}", raw_value));
    }

    Ok(Some(save_path))
}

fn value_contains_upload_reference(value: &Value) -> bool {
    match value {
        Value::String(text) => text.starts_with(UPLOAD_REFERENCE_PREFIX),
        Value::Array(items) => items.iter().any(value_contains_upload_reference),
        Value::Object(entries) => entries.values().any(value_contains_upload_reference),
        _ => false,
    }
}

fn rewrite_upload_references_in_value(value: Value, upload_root: &FsPath) -> Result<Value, String> {
    match value {
        Value::String(text) => {
            if let Some(path) = resolve_upload_reference(upload_root, &text)? {
                Ok(Value::String(path.to_string_lossy().to_string()))
            } else {
                Ok(Value::String(text))
            }
        }
        Value::Array(items) => items
            .into_iter()
            .map(|item| rewrite_upload_references_in_value(item, upload_root))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(entries) => entries
            .into_iter()
            .map(|(key, value)| {
                rewrite_upload_references_in_value(value, upload_root).map(|value| (key, value))
            })
            .collect::<Result<serde_json::Map<String, Value>, _>>()
            .map(Value::Object),
        other => Ok(other),
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
            log_upload_warn(
                "cleanup_saved_files",
                &format!("failed to clean up partial upload '{}' : {}", path.display(), error),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_upload_reference, build_upload_target_path, resolve_upload_reference,
        rewrite_upload_references_in_value,
    };
    use serde_json::{json, Value};

    #[test]
    fn upload_reference_round_trips_to_saved_file_path() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(upload_dir.path()).expect("canonical root");
        let target = build_upload_target_path(&root, "plugin.jar").expect("upload target path");
        std::fs::write(&target, b"plugin").expect("uploaded file should exist");

        let resolved = resolve_upload_reference(&root, &build_upload_reference("plugin.jar"))
            .expect("upload reference should resolve")
            .expect("upload reference should map to a file path");

        assert_eq!(resolved, target);
    }

    #[test]
    fn nested_upload_references_are_rewritten_to_real_paths() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(upload_dir.path()).expect("canonical root");
        let target = build_upload_target_path(&root, "plugin.jar").expect("upload target path");
        std::fs::write(&target, b"plugin").expect("uploaded file should exist");

        let params = json!({
            "path": build_upload_reference("plugin.jar"),
            "nested": {
                "paths": [build_upload_reference("plugin.jar")]
            }
        });

        let rewritten = rewrite_upload_references_in_value(params, &root)
            .expect("upload references should rewrite");

        assert_eq!(rewritten["path"], Value::String(target.to_string_lossy().to_string()));
        assert_eq!(
            rewritten["nested"]["paths"][0],
            Value::String(target.to_string_lossy().to_string())
        );
    }

    #[test]
    fn upload_reference_rejects_missing_files() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(upload_dir.path()).expect("canonical root");

        let error = resolve_upload_reference(&root, &build_upload_reference("missing.jar"))
            .expect_err("missing upload reference should not resolve");

        assert!(
            error.contains("Uploaded file reference not found"),
            "unexpected error: {}",
            error
        );
    }
}
