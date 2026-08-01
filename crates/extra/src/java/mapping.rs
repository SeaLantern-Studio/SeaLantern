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
        confidence: info.confidence,
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
    if value.trim().eq_ignore_ascii_case("unknown") {
        String::new()
    } else {
        value
    }
}

fn parse_major_version(version: &str) -> u32 {
    let mut numbers = version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u32>().ok());
    let Some(first) = numbers.next() else {
        return 0;
    };

    if first == 1 {
        numbers.next().unwrap_or(first)
    } else {
        first
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
        assert_eq!(parse_major_version("jdk-21"), 21);
        assert_eq!(parse_major_version("OpenJDK 21.0.4"), 21);
        assert_eq!(parse_major_version("jdk-1.8.0_402"), 8);
        assert_eq!(parse_major_version(""), 0);
        assert_eq!(parse_major_version("unknown"), 0);
    }

    #[test]
    fn vendor_info_maps_to_project_info() {
        let info = VendorJavaInfo {
            name: "OpenJDK".to_string(),
            version: "21.0.4".to_string(),
            path: PathBuf::from(r"C:\Java\jdk-21\bin\java.exe"),
            vendor: "Eclipse Adoptium".to_string(),
            architecture: "amd64".to_string(),
            confidence: 95,
            java_home: PathBuf::from(r"C:\Java\jdk-21"),
        };

        let app_info = to_app_java_info(info);

        assert_eq!(app_info.version, "21.0.4");
        assert_eq!(app_info.vendor, "Eclipse Adoptium");
        assert_eq!(app_info.major_version, 21);
        assert!(app_info.is_64bit);
        assert_eq!(app_info.confidence, 95);
    }

    #[test]
    fn unknown_metadata_is_exposed_as_empty_values() {
        assert!(normalize_unknown("UNKNOWN".to_string()).is_empty());
        assert!(normalize_unknown(" unknown ".to_string()).is_empty());
        assert!(normalize_unknown("Unknown".to_string()).is_empty());
        assert_eq!(normalize_unknown("OpenJDK".to_string()), "OpenJDK");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_path_strips_windows_extended_prefix() {
        let path = PathBuf::from(r"\\?\C:\Java\jdk-21\bin\java.exe");

        assert_eq!(normalize_path(&path), r"C:\Java\jdk-21\bin\java.exe");
    }
}
