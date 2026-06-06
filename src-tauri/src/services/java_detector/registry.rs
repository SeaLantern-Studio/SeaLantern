use std::path::Path;

use winreg::enums::*;
use winreg::RegKey;

/// 从 Windows 注册表读取 Java 安装路径
pub(super) fn get_javas_from_registry() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut found = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let keys = vec!["SOFTWARE\\JavaSoft", "SOFTWARE\\WOW6432Node\\JavaSoft"];

    for key_path in keys {
        if let Ok(root_key) = hklm.open_subkey(key_path) {
            search_reg_recursive(&root_key, &mut found);
        }
    }

    Ok(found)
}

fn search_reg_recursive(key: &RegKey, results: &mut Vec<String>) {
    if let Ok(home) = key.get_value::<String, _>("JavaHome") {
        let path = Path::new(&home).join("bin").join("java.exe");
        if path.exists() {
            results.push(path.to_string_lossy().into_owned());
        }
    }

    for name in key.enum_keys().flatten() {
        if let Ok(sub) = key.open_subkey(&name) {
            search_reg_recursive(&sub, results);
        }
    }
}
