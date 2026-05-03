//! 插件清单相关的固定提示内容。

use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;

pub fn unsupported_plugin_source_message() -> String {
    format!(
        "不支持的文件格式，请提供 .zip 文件、{} 或插件目录",
        PLUGIN_MANIFEST_FILE_NAME
    )
}

pub fn unsupported_plugin_source_message_en() -> String {
    format!(
        "Unsupported file format. Please provide a .zip file or {}",
        PLUGIN_MANIFEST_FILE_NAME
    )
}

pub fn invalid_manifest_path_message() -> String {
    format!("Invalid {} path", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn missing_manifest_in_folder_message() -> String {
    format!("Folder does not contain {}", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn manifest_not_found_in_dir_message(path: &std::path::Path) -> String {
    format!("{} not found in {}", PLUGIN_MANIFEST_FILE_NAME, path.display())
}

pub fn manifest_not_found_in_zip_message() -> String {
    format!("{} not found in ZIP archive", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn read_manifest_failed_message(error: &dyn std::fmt::Display) -> String {
    format!("Failed to read {}: {}", PLUGIN_MANIFEST_FILE_NAME, error)
}

pub fn parse_manifest_failed_message(error: &dyn std::fmt::Display) -> String {
    format!("Failed to parse {}: {}", PLUGIN_MANIFEST_FILE_NAME, error)
}

pub fn writing_manifest_not_allowed_message() -> String {
    format!("Writing {} is not allowed", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn missing_permission_in_manifest_message(module_name: &str) -> String {
    format!(
        "权限不足: 使用 'sl.{}' 模块需要在 {} 中声明 '{}' 权限",
        module_name, PLUGIN_MANIFEST_FILE_NAME, module_name
    )
}
