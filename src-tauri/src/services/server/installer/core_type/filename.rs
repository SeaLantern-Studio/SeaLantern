use super::definitions::CoreType;
use std::path::Path;

pub fn detect_core_type(input: &str) -> String {
    let path = Path::new(input);
    let target_file = if super::super::archive::is_script_file(path) {
        path.parent()
            .and_then(super::super::archive::find_server_jar_in_dir)
            .unwrap_or_else(|| input.to_string())
    } else {
        path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string())
    };

    CoreType::detect_from_filename(&target_file).to_string()
}
