use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;

use crate::models::server::ParsedServerCoreInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreType {
    ArclightForge,
    ArclightNeoforge,
    Youer,
    Mohist,
    Catserver,
    Spongeforge,
    ArclightFabric,
    Banner,
    Neoforge,
    Forge,
    Quilt,
    Fabric,
    PufferfishPurpur,
    Pufferfish,
    Spongevanilla,
    Purpur,
    Paper,
    Folia,
    Leaves,
    Leaf,
    Spigot,
    Bukkit,
    VanillaSnapshot,
    Vanilla,
    Nukkitx,
    Bedrock,
    Velocity,
    Bungeecord,
    Lightfall,
    Travertine,
    Unknown,
}

impl CoreType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoreType::ArclightForge => "Arclight-Forge",
            CoreType::ArclightNeoforge => "Arclight-Neoforge",
            CoreType::Youer => "Youer",
            CoreType::Mohist => "Mohist",
            CoreType::Catserver => "Catserver",
            CoreType::Spongeforge => "Spongeforge",
            CoreType::ArclightFabric => "Arclight-Fabric",
            CoreType::Banner => "Banner",
            CoreType::Neoforge => "Neoforge",
            CoreType::Forge => "Forge",
            CoreType::Quilt => "Quilt",
            CoreType::Fabric => "Fabric",
            CoreType::PufferfishPurpur => "Pufferfish_Purpur",
            CoreType::Pufferfish => "Pufferfish",
            CoreType::Spongevanilla => "Spongevanilla",
            CoreType::Purpur => "Purpur",
            CoreType::Paper => "Paper",
            CoreType::Folia => "Folia",
            CoreType::Leaves => "Leaves",
            CoreType::Leaf => "Leaf",
            CoreType::Spigot => "Spigot",
            CoreType::Bukkit => "Bukkit",
            CoreType::VanillaSnapshot => "Vanilla-Snapshot",
            CoreType::Vanilla => "Vanilla",
            CoreType::Nukkitx => "Nukkitx",
            CoreType::Bedrock => "Bedrock",
            CoreType::Velocity => "Velocity",
            CoreType::Bungeecord => "Bungeecord",
            CoreType::Lightfall => "Lightfall",
            CoreType::Travertine => "Travertine",
            CoreType::Unknown => "Unknown",
        }
    }

    fn detection_table() -> &'static [(CoreType, &'static [&'static str])] {
        &[
            (CoreType::ArclightForge, &["arclight-forge"]),
            (CoreType::ArclightNeoforge, &["arclight-neoforge"]),
            (CoreType::Youer, &["youer"]),
            (CoreType::Mohist, &["mohist"]),
            (CoreType::Catserver, &["catserver"]),
            (CoreType::Spongeforge, &["spongeforge"]),
            (CoreType::ArclightFabric, &["arclight-fabric"]),
            (CoreType::Banner, &["banner"]),
            (CoreType::Neoforge, &["neoforge"]),
            (CoreType::Forge, &["forge"]),
            (CoreType::Quilt, &["quilt"]),
            (CoreType::Fabric, &["fabric"]),
            (CoreType::PufferfishPurpur, &["pufferfish_purpur", "pufferfish-purpur"]),
            (CoreType::Pufferfish, &["pufferfish"]),
            (CoreType::Spongevanilla, &["spongevanilla"]),
            (CoreType::Purpur, &["purpur"]),
            (CoreType::Paper, &["paper"]),
            (CoreType::Folia, &["folia"]),
            (CoreType::Leaves, &["leaves"]),
            (CoreType::Leaf, &["leaf"]),
            (CoreType::Spigot, &["spigot"]),
            (CoreType::Bukkit, &["bukkit"]),
            (CoreType::VanillaSnapshot, &["vanilla-snapshot"]),
            (CoreType::Vanilla, &["vanilla"]),
            (CoreType::Nukkitx, &["nukkitx", "nukkit"]),
            (CoreType::Bedrock, &["bedrock"]),
            (CoreType::Velocity, &["velocity"]),
            (CoreType::Bungeecord, &["bungeecord"]),
            (CoreType::Lightfall, &["lightfall"]),
            (CoreType::Travertine, &["travertine"]),
        ]
    }

    pub fn detect_from_filename(filename: &str) -> Self {
        let filename_lower = filename.to_lowercase();
        for (core_type, keywords) in Self::detection_table() {
            for keyword in *keywords {
                if filename_lower.contains(keyword) {
                    return *core_type;
                }
            }
        }
        CoreType::Unknown
    }
}

impl FromStr for CoreType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "arclight-forge" => Ok(CoreType::ArclightForge),
            "arclight-neoforge" => Ok(CoreType::ArclightNeoforge),
            "youer" => Ok(CoreType::Youer),
            "mohist" => Ok(CoreType::Mohist),
            "catserver" => Ok(CoreType::Catserver),
            "spongeforge" => Ok(CoreType::Spongeforge),
            "arclight-fabric" => Ok(CoreType::ArclightFabric),
            "banner" => Ok(CoreType::Banner),
            "neoforge" => Ok(CoreType::Neoforge),
            "forge" => Ok(CoreType::Forge),
            "quilt" => Ok(CoreType::Quilt),
            "fabric" => Ok(CoreType::Fabric),
            "pufferfish_purpur" | "pufferfish-purpur" => Ok(CoreType::PufferfishPurpur),
            "pufferfish" => Ok(CoreType::Pufferfish),
            "spongevanilla" => Ok(CoreType::Spongevanilla),
            "purpur" => Ok(CoreType::Purpur),
            "paper" => Ok(CoreType::Paper),
            "folia" => Ok(CoreType::Folia),
            "leaves" => Ok(CoreType::Leaves),
            "leaf" => Ok(CoreType::Leaf),
            "spigot" => Ok(CoreType::Spigot),
            "bukkit" => Ok(CoreType::Bukkit),
            "vanilla-snapshot" => Ok(CoreType::VanillaSnapshot),
            "vanilla" => Ok(CoreType::Vanilla),
            "nukkitx" | "nukkit" => Ok(CoreType::Nukkitx),
            "bedrock" => Ok(CoreType::Bedrock),
            "velocity" => Ok(CoreType::Velocity),
            "bungeecord" => Ok(CoreType::Bungeecord),
            "lightfall" => Ok(CoreType::Lightfall),
            "travertine" => Ok(CoreType::Travertine),
            "unknown" => Ok(CoreType::Unknown),
            _ => Err(format!("Unknown core type: {}", s)),
        }
    }
}

impl std::fmt::Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn detect_core_type(input: &str) -> String {
    let path = Path::new(input);
    let target_file = if is_script_file(path) {
        path.parent()
            .and_then(find_server_jar_in_dir)
            .unwrap_or_else(|| input.to_string())
    } else {
        path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string())
    };

    CoreType::detect_from_filename(&target_file).to_string()
}

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

pub fn extract_modpack_archive(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    let lower_name = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase())
        .unwrap_or_default();

    if lower_name.ends_with(".zip") {
        let file =
            std::fs::File::open(archive_path).map_err(|e| format!("无法打开压缩包文件: {}", e))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("无法解析 ZIP 压缩包: {}", e))?;
        return extract_zip_archive(&mut archive, target_dir);
    }

    if lower_name.ends_with(".tar.gz") || lower_name.ends_with(".tgz") {
        let file =
            std::fs::File::open(archive_path).map_err(|e| format!("无法打开压缩包文件: {}", e))?;
        let decoder = GzDecoder::new(file);
        return extract_tar_archive(decoder, target_dir);
    }

    if lower_name.ends_with(".tar") {
        let file =
            std::fs::File::open(archive_path).map_err(|e| format!("无法打开压缩包文件: {}", e))?;
        return extract_tar_archive(file, target_dir);
    }

    Err("暂不支持该压缩包格式，仅支持 .zip、.tar、.tar.gz、.tgz".to_string())
}

pub fn resolve_extracted_root(extract_dir: &Path) -> PathBuf {
    let entries = match std::fs::read_dir(extract_dir) {
        Ok(entries) => entries,
        Err(_) => return extract_dir.to_path_buf(),
    };

    let mut directories = Vec::new();
    let mut file_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else {
            file_count += 1;
        }
    }

    if file_count == 0 && directories.len() == 1 {
        return directories.remove(0);
    }

    extract_dir.to_path_buf()
}

pub fn find_server_jar(modpack_path: &Path) -> Result<String, String> {
    let patterns = vec![
        "server.jar",
        "forge.jar",
        "fabric-server.jar",
        "minecraft_server.jar",
        "paper.jar",
        "spigot.jar",
        "purpur.jar",
    ];

    for pattern in &patterns {
        let jar_path = modpack_path.join(pattern);
        if jar_path.exists() {
            return Ok(jar_path.to_string_lossy().to_string());
        }
    }

    let entries = std::fs::read_dir(modpack_path).map_err(|e| format!("无法读取文件夹: {}", e))?;
    let mut jar_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jar" {
                    jar_files.push(path);
                }
            }
        }
    }

    if jar_files.is_empty() {
        return Err("整合包文件夹中未找到JAR文件".to_string());
    }

    if jar_files.len() == 1 {
        return Ok(jar_files[0].to_string_lossy().to_string());
    }

    for jar in &jar_files {
        if let Some(name) = jar.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();
            if name_lower.contains("server")
                || name_lower.contains("forge")
                || name_lower.contains("fabric")
                || name_lower.contains("mohist")
                || name_lower.contains("paper")
                || name_lower.contains("spigot")
                || name_lower.contains("purpur")
                || name_lower.contains("bukkit")
                || name_lower.contains("catserver")
                || name_lower.contains("arclight")
            {
                return Ok(jar.to_string_lossy().to_string());
            }
        }
    }

    Ok(jar_files[0].to_string_lossy().to_string())
}

fn detect_core_type_with_main_class(input: &str) -> (String, Option<String>) {
    let main_class = read_jar_main_class(input);
    if let Some(ref class_name) = main_class {
        if let Some(core_type) = core_type_from_main_class(class_name) {
            return (core_type.to_string(), Some(class_name.clone()));
        }
    }
    (detect_core_type(input), main_class)
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

fn read_jar_main_class(jar_path: &str) -> Option<String> {
    let file = std::fs::File::open(jar_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    let mut manifest = archive.by_name("META-INF/MANIFEST.MF").ok()?;

    let mut bytes = Vec::new();
    manifest.read_to_end(&mut bytes).ok()?;
    let content = String::from_utf8_lossy(&bytes).to_string();

    find_manifest_main_class(&content)
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

fn is_script_file(path: &Path) -> bool {
    path.extension()
        .map(|e| {
            let ext = e.to_string_lossy().to_lowercase();
            ext == "sh" || ext == "bat"
        })
        .unwrap_or(false)
}

fn find_server_jar_in_dir(dir: &Path) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension()? == "jar" {
                path.file_name().map(|n| n.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .next()
}

fn extract_zip_archive(
    archive: &mut ZipArchive<std::fs::File>,
    target_dir: &Path,
) -> Result<(), String> {
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let enclosed_path = file
            .enclosed_name()
            .ok_or_else(|| "ZIP 条目包含非法路径".to_string())?;
        let out_path = target_dir.join(enclosed_path);

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let mut out_file =
            std::fs::File::create(&out_path).map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut file, &mut out_file).map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(())
}

fn extract_tar_archive<R: Read>(reader: R, target_dir: &Path) -> Result<(), String> {
    let mut archive = Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|e| format!("读取 TAR 条目失败: {}", e))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| format!("解析 TAR 条目失败: {}", e))?;
        entry
            .unpack_in(target_dir)
            .map_err(|e| format!("解压 TAR 条目失败: {}", e))?;
    }

    Ok(())
}
