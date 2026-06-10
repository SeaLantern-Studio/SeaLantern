use std::path::{Path, PathBuf};

use crate::archive::{
    extract_modpack_archive, find_server_jar_checked, resolve_extracted_root_checked,
};
use crate::core_type::CoreType;
use crate::{detect_core_key_checked, detect_core_type_with_main_class_checked};

struct TempExtractDir(PathBuf);

impl TempExtractDir {
    fn new(prefix: &str) -> Result<Self, String> {
        let path = std::env::temp_dir().join(format!("{}_{}", prefix, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).map_err(|e| format!("无法创建临时解压目录: {}", e))?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempExtractDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedServerCoreInfo {
    pub core_type: String,
    pub main_class: Option<String>,
    pub jar_path: Option<String>,
}

pub fn parse_server_core_type(source_path: &str) -> Result<ParsedServerCoreInfo, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("路径不存在: {}", source_path));
    }

    let mut extracted_temp_dir: Option<TempExtractDir> = None;

    let detected_jar = if source.is_dir() {
        match find_server_jar_checked(source) {
            Ok(path) => Some(PathBuf::from(path)),
            Err(error) if error == "整合包文件夹中未找到JAR文件" => None,
            Err(error) => {
                return Err(format!("解析服务器核心类型失败: {}", error));
            }
        }
    } else if source.is_file() {
        let source_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase())
            .unwrap_or_default();

        if source_name.ends_with(".jar") {
            Some(source.to_path_buf())
        } else {
            let temp_dir = TempExtractDir::new("sea_lantern_core_detect")?;
            extract_modpack_archive(source, temp_dir.path())?;
            let root_dir = resolve_extracted_root_checked(temp_dir.path())
                .map_err(|error| format!("解析服务器核心类型失败: {}", error))?;
            let detected = match find_server_jar_checked(&root_dir) {
                Ok(path) => Some(PathBuf::from(path)),
                Err(error) if error == "整合包文件夹中未找到JAR文件" => None,
                Err(error) => {
                    return Err(format!("解析服务器核心类型失败: {}", error));
                }
            };
            extracted_temp_dir = Some(temp_dir);
            detected
        }
    } else {
        None
    };

    let result = if let Some(jar_path) = detected_jar {
        let jar_text = jar_path.to_string_lossy().to_string();
        let (core_type, main_class) = detect_core_type_with_main_class_checked(&jar_text)
            .map_err(|error| format!("解析服务器核心类型失败: {}", error))?;
        let stable_jar_path = if let Some(temp_dir) = extracted_temp_dir.as_ref() {
            Some(to_relative_archive_path(temp_dir.path(), &jar_path)?)
        } else {
            Some(jar_text)
        };

        ParsedServerCoreInfo {
            core_type,
            main_class,
            jar_path: stable_jar_path,
        }
    } else {
        ParsedServerCoreInfo {
            core_type: CoreType::Unknown.to_string(),
            main_class: None,
            jar_path: None,
        }
    };

    Ok(result)
}

pub fn parse_server_core_key(source_path: &str) -> Result<ParsedServerCoreInfo, String> {
    let mut parsed = parse_server_core_type(source_path)?;
    if let Some(jar_path) = parsed.jar_path.as_deref() {
        if let Ok(core_key) = detect_core_key_checked(jar_path) {
            if core_key.eq_ignore_ascii_case("unknown") {
                parsed.core_type = CoreType::normalize_to_api_core_key(&parsed.core_type)
                    .unwrap_or(parsed.core_type);
                return Ok(parsed);
            }
            parsed.core_type = core_key;
            return Ok(parsed);
        }
    }

    parsed.core_type =
        CoreType::normalize_to_api_core_key(&parsed.core_type).unwrap_or(parsed.core_type);
    Ok(parsed)
}

fn to_relative_archive_path(base_dir: &Path, absolute_path: &Path) -> Result<String, String> {
    let relative = absolute_path
        .strip_prefix(base_dir)
        .map_err(|_| format!("扫描到的启动文件不在临时解压目录内: {}", absolute_path.display()))?;

    if relative.as_os_str().is_empty() {
        return Err("扫描到的启动文件路径无效".to_string());
    }

    Ok(relative.to_string_lossy().to_string())
}
