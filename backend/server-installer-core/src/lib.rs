mod archive;
mod core_type;
mod mc_version;
mod parser;

use std::path::Path;

pub use archive::{
    extract_modpack_archive, find_server_jar, find_server_jar_checked, resolve_extracted_root,
    resolve_extracted_root_checked,
};
pub use core_type::CoreType;
pub use parser::{parse_server_core_type, ParsedServerCoreInfo};

pub fn detect_core_type(input: &str) -> String {
    detect_core_type_checked(input).unwrap_or_else(|_| {
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

pub fn detect_mc_version_from_mods(root_dir: &Path, known_versions: &[&str]) -> (Option<String>, bool) {
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
    let main_class = read_jar_main_class_checked(input)?;
    if let Some(ref class_name) = main_class {
        if let Some(core_type) = core_type_from_main_class(class_name) {
            return Ok((core_type.to_string(), Some(class_name.clone())));
        }
    }
    Ok((detect_core_type_checked(input)?, main_class))
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
    let file = std::fs::File::open(jar_path)
        .map_err(|e| format!("读取 JAR 文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析 JAR 压缩结构失败: {}", e))?;
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
