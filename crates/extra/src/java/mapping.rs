use std::{collections::HashSet, path::Path};

use java_manager::JavaInfo as VendorJavaInfo;

use crate::config::JavaInfo;

pub(crate) fn push_unique(
    results: &mut Vec<JavaInfo>,
    seen: &mut HashSet<String>,
    info: VendorJavaInfo,
) {
    let app_info = to_app_java_info(info);
    let key = normalize_path_key(&app_info.path);
    if seen.insert(key) {
        results.push(app_info);
    }
}

pub(crate) fn to_app_java_info(info: VendorJavaInfo) -> JavaInfo {
    let version = normalize_unknown(info.version);
    let vendor = normalize_unknown(info.vendor);
    let architecture = info.architecture.to_ascii_lowercase();
    let major_version = parse_major_version(&version);

    JavaInfo {
        path: normalize_path(&info.path),
        version,
        vendor,
        is_64bit: architecture.contains("64")
            || matches!(architecture.as_str(), "amd64" | "x86_64" | "aarch64"),
        major_version,
    }
}

fn normalize_path(path: &Path) -> String {
    let path = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        path.strip_prefix(r"\\?\").unwrap_or(&path).to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.into_owned()
    }
}

fn normalize_path_key(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        path.to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string()
    }
}

fn normalize_unknown(value: String) -> String {
    if value == "UNKNOWN" {
        String::new()
    } else {
        value
    }
}

fn parse_major_version(version: &str) -> u32 {
    let Some(first) = version.split(['.', '-']).next() else {
        return 0;
    };

    let parsed = first.parse().unwrap_or(0);
    if parsed == 1 {
        version
            .split(['.', '-'])
            .nth(1)
            .and_then(|part| part.parse().ok())
            .unwrap_or(parsed)
    } else {
        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_major_version_handles_legacy_and_modern_versions() {
        assert_eq!(parse_major_version("1.8.0_402"), 8);
        assert_eq!(parse_major_version("17.0.12"), 17);
        assert_eq!(parse_major_version("21.0.4-LTS"), 21);
        assert_eq!(parse_major_version(""), 0);
    }

    #[test]
    fn vendor_info_maps_to_project_info() {
        let info = VendorJavaInfo {
            name: "OpenJDK".to_string(),
            version: "21.0.4".to_string(),
            path: PathBuf::from(r"C:\Java\jdk-21\bin\java.exe"),
            vendor: "Eclipse Adoptium".to_string(),
            architecture: "amd64".to_string(),
            java_home: PathBuf::from(r"C:\Java\jdk-21"),
        };

        let app_info = to_app_java_info(info);

        assert_eq!(app_info.version, "21.0.4");
        assert_eq!(app_info.vendor, "Eclipse Adoptium");
        assert_eq!(app_info.major_version, 21);
        assert!(app_info.is_64bit);
    }

    #[test]
    fn unknown_metadata_is_exposed_as_empty_values() {
        assert!(normalize_unknown("UNKNOWN".to_string()).is_empty());
        assert_eq!(normalize_unknown("OpenJDK".to_string()), "OpenJDK");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_path_strips_windows_extended_prefix() {
        let path = PathBuf::from(r"\\?\C:\Java\jdk-21\bin\java.exe");

        assert_eq!(normalize_path(&path), r"C:\Java\jdk-21\bin\java.exe");
    }
}
