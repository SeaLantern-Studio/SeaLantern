use super::archive::{extract_modpack_archive, find_server_jar, resolve_extracted_root};
use super::core_type::{detect_core_type_with_main_class, CoreType};
use crate::models::server::ParsedServerCoreInfo;
use std::path::{Path, PathBuf};

pub fn parse_server_core_type(source_path: &str) -> Result<ParsedServerCoreInfo, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("路径不存在: {}", source_path));
    }

    let mut extracted_temp_dir: Option<PathBuf> = None;

    let detected_jar = if source.is_dir() {
        find_server_jar(source).ok().map(PathBuf::from)
    } else if source.is_file() {
        let source_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase())
            .unwrap_or_default();

        if source_name.ends_with(".jar") {
            Some(source.to_path_buf())
        } else {
            let temp_dir = std::env::temp_dir()
                .join(format!("sea_lantern_core_detect_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("无法创建临时解压目录: {}", e))?;

            match extract_modpack_archive(source, &temp_dir) {
                Ok(()) => {
                    let root_dir = resolve_extracted_root(&temp_dir);
                    extracted_temp_dir = Some(temp_dir);
                    find_server_jar(&root_dir).ok().map(PathBuf::from)
                }
                Err(_) => {
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    None
                }
            }
        }
    } else {
        None
    };

    let result = if let Some(jar_path) = detected_jar {
        let jar_text = jar_path.to_string_lossy().to_string();
        let (core_type, main_class) = detect_core_type_with_main_class(&jar_text);
        ParsedServerCoreInfo {
            core_type,
            main_class,
            jar_path: Some(jar_text),
        }
    } else {
        ParsedServerCoreInfo {
            core_type: CoreType::Unknown.to_string(),
            main_class: None,
            jar_path: None,
        }
    };

    if let Some(temp_dir) = extracted_temp_dir {
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    Ok(result)
}
