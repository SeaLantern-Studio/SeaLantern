mod archive;
mod core_type;
mod mc_version;
mod parser;

use std::path::Path;
use std::str::FromStr;

pub use archive::{
    extract_modpack_archive, find_server_jar, find_server_jar_checked, resolve_extracted_root,
    resolve_extracted_root_checked,
};
pub use core_type::{CoreType, DockerTypeResolution, StarterInstallArgs, StarterInstallMode};
pub use parser::{parse_server_core_key, parse_server_core_type, ParsedServerCoreInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarterCoreKeyResolution {
    pub starter_core_key: String,
    pub detected_core_key: String,
}

impl StarterCoreKeyResolution {
    pub fn is_starter_mode(&self) -> bool {
        !self.starter_core_key.is_empty() || !self.detected_core_key.is_empty()
    }

    pub fn needs_unrecognized_error(&self, requested_startup_mode: &str) -> bool {
        requested_startup_mode
            .trim()
            .eq_ignore_ascii_case("starter")
            && self.starter_core_key.is_empty()
    }

    pub fn unresolved_display_hint(&self, explicit_core_type: Option<&str>) -> String {
        explicit_core_type
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| self.detected_core_key.clone())
    }
}

fn normalize_detected_core_type(raw: &str) -> String {
    CoreType::normalize_to_api_core_key(raw).unwrap_or_else(|| raw.to_string())
}

pub fn detect_core_type(input: &str) -> String {
    detect_core_type_checked(input).unwrap_or_else(|_| {
        let path = Path::new(input);
        path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string())
    })
}

pub fn detect_core_key(input: &str) -> String {
    detect_core_key_checked(input).unwrap_or_else(|_| {
        let path = Path::new(input);
        path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string())
    })
}

pub fn detect_core_type_checked(input: &str) -> Result<String, String> {
    let path = Path::new(input);
    let target_file = if archive::is_script_file(path) {
        match path.parent() {
            Some(parent) => archive::find_server_jar_in_dir_checked(parent)?
                .unwrap_or_else(|| input.to_string()),
            None => input.to_string(),
        }
    } else {
        path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string())
    };

    Ok(CoreType::detect_from_filename(&target_file).to_string())
}

pub fn detect_core_key_checked(input: &str) -> Result<String, String> {
    detect_core_type_checked(input).map(|value| normalize_detected_core_type(&value))
}

pub fn resolve_starter_core_key(
    explicit_core_type: Option<&str>,
    startup_target_path: Option<&str>,
) -> Option<String> {
    explicit_core_type
        .and_then(CoreType::normalize_to_api_core_key)
        .or_else(|| {
            startup_target_path
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .and_then(|path| detect_core_key_checked(path).ok())
                .and_then(|detected| CoreType::normalize_to_api_core_key(&detected))
        })
}

pub fn resolve_starter_core_key_checked(
    requested_startup_mode: &str,
    explicit_core_type: Option<&str>,
    startup_target_path: Option<&str>,
) -> StarterCoreKeyResolution {
    if !requested_startup_mode
        .trim()
        .eq_ignore_ascii_case("starter")
    {
        return StarterCoreKeyResolution {
            starter_core_key: String::new(),
            detected_core_key: String::new(),
        };
    }

    let detected_core_key = startup_target_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|path| detect_core_key_checked(path).ok())
        .unwrap_or_default();
    let starter_core_key =
        resolve_starter_core_key(explicit_core_type, startup_target_path).unwrap_or_default();

    StarterCoreKeyResolution { starter_core_key, detected_core_key }
}

pub fn resolve_imported_server_core_key(startup_mode: &str, startup_target_path: &str) -> String {
    if startup_mode.trim().eq_ignore_ascii_case("custom") {
        let normalized = startup_target_path.trim().to_ascii_lowercase();
        if normalized.contains("pumpkin") {
            return "pumpkin".to_string();
        }
        return "custom".to_string();
    }

    normalize_detected_core_type(&detect_core_key(startup_target_path))
}

pub fn should_copy_modpack_source_as_native_server_binary(source_path: &Path) -> bool {
    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();
    if extension == "exe" {
        return true;
    }

    extension.is_empty()
        && source_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| detect_core_key(name) == "pumpkin")
}

pub fn should_delay_starter_runtime_file_writes(
    requested_startup_mode: &str,
    source_path: &Path,
) -> bool {
    requested_startup_mode.eq_ignore_ascii_case("starter")
        && source_path.is_file()
        && source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
}

pub fn detect_mc_version_from_mods(
    root_dir: &Path,
    known_versions: &[&str],
) -> (Option<String>, bool) {
    mc_version::detect_mc_version_from_mods(root_dir, known_versions)
}

pub fn detect_mc_version_from_mods_checked(
    root_dir: &Path,
    known_versions: &[&str],
) -> Result<(Option<String>, bool), String> {
    mc_version::detect_mc_version_from_mods_checked(root_dir, known_versions)
}

fn detect_core_type_with_main_class_checked(
    input: &str,
) -> Result<(String, Option<String>), String> {
    let filename_core = detect_core_type_checked(input)?;
    let main_class = read_jar_main_class_checked(input)?;
    if let Some(ref class_name) = main_class {
        if let Some(core_type) = core_type_from_main_class(class_name) {
            let resolved_core_type = reconcile_main_class_and_filename_core_type(
                CoreType::from_str(&filename_core).unwrap_or(CoreType::Unknown),
                core_type,
            );
            return Ok((resolved_core_type.to_string(), Some(class_name.clone())));
        }
    }
    Ok((filename_core, main_class))
}

fn reconcile_main_class_and_filename_core_type(
    filename_core: CoreType,
    main_class_core: CoreType,
) -> CoreType {
    match (filename_core, main_class_core) {
        // NeoForge installers can still expose the legacy Forge SimpleInstaller main class.
        (CoreType::Neoforge, CoreType::Forge) | (CoreType::ArclightNeoforge, CoreType::Forge) => {
            filename_core
        }
        (_, main_class_core) => main_class_core,
    }
}

fn core_type_from_main_class(main_class: &str) -> Option<CoreType> {
    match main_class {
        value if value.starts_with("net.neoforged.serverstarterjar") => Some(CoreType::Neoforge),
        "net.minecraft.server.MinecraftServer" | "net.minecraft.bundler.Main" => {
            Some(CoreType::Vanilla)
        }
        "net.minecraft.client.Main" => Some(CoreType::Unknown),
        "net.minecraftforge.installer.SimpleInstaller" => Some(CoreType::Forge),
        "net.fabricmc.installer.Main" => Some(CoreType::Fabric),
        "net.fabricmc.installer.ServerLauncher" => Some(CoreType::Fabric),
        "io.izzel.arclight.server.Launcher" => Some(CoreType::ArclightForge),
        "catserver.server.CatServerLaunch" | "foxlaunch.FoxServerLauncher" => {
            Some(CoreType::Catserver)
        }
        "org.bukkit.craftbukkit.Main" | "org.bukkit.craftbukkit.bootstrap.Main" => {
            Some(CoreType::Bukkit)
        }
        "io.papermc.paperclip.Main" | "com.destroystokyo.paperclip.Paperclip" => {
            Some(CoreType::Paper)
        }
        "org.leavesmc.leavesclip.Main" => Some(CoreType::Leaves),
        "net.md_5.bungee.Bootstrap" => Some(CoreType::Lightfall),
        "com.mohistmc.MohistMCStart" | "com.mohistmc.MohistMC" => Some(CoreType::Mohist),
        "com.velocitypowered.proxy.Velocity" => Some(CoreType::Velocity),
        _ => None,
    }
}

fn read_jar_main_class_checked(jar_path: &str) -> Result<Option<String>, String> {
    let file = std::fs::File::open(jar_path).map_err(|e| format!("读取 JAR 文件失败: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("解析 JAR 压缩结构失败: {}", e))?;
    let mut manifest = archive
        .by_name("META-INF/MANIFEST.MF")
        .map_err(|e| format!("读取 JAR manifest 失败: {}", e))?;

    let mut bytes = Vec::new();
    use std::io::Read;
    manifest
        .read_to_end(&mut bytes)
        .map_err(|e| format!("读取 JAR manifest 内容失败: {}", e))?;
    let content = String::from_utf8_lossy(&bytes).to_string();

    Ok(find_manifest_main_class(&content))
}

fn find_manifest_main_class(manifest_content: &str) -> Option<String> {
    let mut current_key = String::new();
    let mut current_value = String::new();

    let flush_entry = |key: &str, value: &str| {
        if key.eq_ignore_ascii_case("Main-Class") {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        }
    };

    for line in manifest_content.lines() {
        if line.is_empty() {
            if let Some(value) = flush_entry(&current_key, &current_value) {
                return Some(value);
            }
            current_key.clear();
            current_value.clear();
            continue;
        }

        if line.starts_with(' ') {
            current_value.push_str(line.trim_start());
            continue;
        }

        if let Some(value) = flush_entry(&current_key, &current_value) {
            return Some(value);
        }

        if let Some((key, value)) = line.split_once(':') {
            current_key.clear();
            current_key.push_str(key.trim());
            current_value.clear();
            current_value.push_str(value.trim());
        } else {
            current_key.clear();
            current_value.clear();
        }
    }

    flush_entry(&current_key, &current_value)
}
