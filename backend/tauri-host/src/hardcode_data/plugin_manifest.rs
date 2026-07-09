//! 插件清单相关的固定提示内容。

use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::plugins::manager::i18n::{plugin_t1, plugin_t2, plugin_t3};

pub fn unsupported_plugin_source_message() -> String {
    plugin_t1("plugin.install.unsupported_source", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn invalid_manifest_path_message() -> String {
    plugin_t1("plugin.install.invalid_manifest_path", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn missing_manifest_in_folder_message() -> String {
    plugin_t1("plugin.install.missing_manifest_in_folder", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn manifest_not_found_in_dir_message(path: &std::path::Path) -> String {
    plugin_t2(
        "plugin.install.manifest_not_found_in_dir",
        PLUGIN_MANIFEST_FILE_NAME,
        path.display().to_string(),
    )
}

pub fn manifest_not_found_in_zip_message() -> String {
    plugin_t1("plugin.install.manifest_not_found_in_zip", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn read_manifest_failed_message(error: &dyn std::fmt::Display) -> String {
    plugin_t2(
        "plugin.install.read_manifest_failed",
        PLUGIN_MANIFEST_FILE_NAME,
        error.to_string(),
    )
}

pub fn parse_manifest_failed_message(error: &dyn std::fmt::Display) -> String {
    plugin_t2(
        "plugin.install.parse_manifest_failed",
        PLUGIN_MANIFEST_FILE_NAME,
        error.to_string(),
    )
}

pub fn writing_manifest_not_allowed_message() -> String {
    plugin_t1("plugin.install.write_manifest_not_allowed", PLUGIN_MANIFEST_FILE_NAME)
}

pub fn missing_permission_in_manifest_message(module_name: &str) -> String {
    plugin_t3(
        "plugin.permission.missing_in_manifest",
        module_name,
        PLUGIN_MANIFEST_FILE_NAME,
        module_name,
    )
}
