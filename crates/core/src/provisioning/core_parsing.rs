use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use zip::result::ZipError;

/// 一个可识别的服务端核心系列。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreKind {
    ArclightForge,
    ArclightNeoForge,
    Youer,
    Mohist,
    CatServer,
    SpongeForge,
    ArclightFabric,
    Banner,
    NeoForge,
    Forge,
    Quilt,
    Fabric,
    PufferfishPurpur,
    Pufferfish,
    SpongeVanilla,
    Purpur,
    Paper,
    Folia,
    Leaves,
    Leaf,
    Spigot,
    Bukkit,
    VanillaSnapshot,
    Vanilla,
    NukkitX,
    Bedrock,
    Velocity,
    BungeeCord,
    Lightfall,
    Travertine,
    Pumpkin,
    Unknown,
}

impl CoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArclightForge => "arclight-forge",
            Self::ArclightNeoForge => "arclight-neoforge",
            Self::Youer => "youer",
            Self::Mohist => "mohist",
            Self::CatServer => "catserver",
            Self::SpongeForge => "spongeforge",
            Self::ArclightFabric => "arclight-fabric",
            Self::Banner => "banner",
            Self::NeoForge => "neoforge",
            Self::Forge => "forge",
            Self::Quilt => "quilt",
            Self::Fabric => "fabric",
            Self::PufferfishPurpur => "pufferfish_purpur",
            Self::Pufferfish => "pufferfish",
            Self::SpongeVanilla => "spongevanilla",
            Self::Purpur => "purpur",
            Self::Paper => "paper",
            Self::Folia => "folia",
            Self::Leaves => "leaves",
            Self::Leaf => "leaf",
            Self::Spigot => "spigot",
            Self::Bukkit => "bukkit",
            Self::VanillaSnapshot => "vanilla-snapshot",
            Self::Vanilla => "vanilla",
            Self::NukkitX => "nukkitx",
            Self::Bedrock => "bedrock",
            Self::Velocity => "velocity",
            Self::BungeeCord => "bungeecord",
            Self::Lightfall => "lightfall",
            Self::Travertine => "travertine",
            Self::Pumpkin => "pumpkin",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        let filename = filename.to_ascii_lowercase();
        for (kind, keywords) in CORE_KEYWORDS {
            if keywords.iter().any(|keyword| filename.contains(keyword)) {
                return *kind;
            }
        }
        Self::Unknown
    }
}

const CORE_KEYWORDS: &[(CoreKind, &[&str])] = &[
    (CoreKind::ArclightForge, &["arclight-forge"]),
    (CoreKind::ArclightNeoForge, &["arclight-neoforge"]),
    (CoreKind::ArclightFabric, &["arclight-fabric"]),
    (CoreKind::PufferfishPurpur, &["pufferfish_purpur", "pufferfish-purpur"]),
    (CoreKind::VanillaSnapshot, &["vanilla-snapshot"]),
    (CoreKind::Youer, &["youer"]),
    (CoreKind::Mohist, &["mohist"]),
    (CoreKind::CatServer, &["catserver"]),
    (CoreKind::SpongeForge, &["spongeforge"]),
    (CoreKind::Banner, &["banner"]),
    (CoreKind::NeoForge, &["neoforge"]),
    (CoreKind::Forge, &["forge"]),
    (CoreKind::Quilt, &["quilt"]),
    (CoreKind::Fabric, &["fabric"]),
    (CoreKind::Pufferfish, &["pufferfish"]),
    (CoreKind::SpongeVanilla, &["spongevanilla"]),
    (CoreKind::Purpur, &["purpur"]),
    (CoreKind::Paper, &["paper"]),
    (CoreKind::Folia, &["folia"]),
    (CoreKind::Leaves, &["leaves"]),
    (CoreKind::Leaf, &["leaf"]),
    (CoreKind::Spigot, &["spigot"]),
    (CoreKind::Bukkit, &["bukkit"]),
    (CoreKind::NukkitX, &["nukkitx", "nukkit"]),
    (CoreKind::Bedrock, &["bedrock"]),
    (CoreKind::Velocity, &["velocity"]),
    (CoreKind::BungeeCord, &["bungeecord"]),
    (CoreKind::Lightfall, &["lightfall"]),
    (CoreKind::Travertine, &["travertine"]),
    (CoreKind::Pumpkin, &["pumpkin"]),
    (CoreKind::Vanilla, &["vanilla"]),
];

/// 从文件名或 JAR 清单解析的服务端核心元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFileInfo {
    pub kind: CoreKind,
    pub core_version: Option<String>,
    pub minecraft_version: Option<String>,
    pub main_class: Option<String>,
}

impl CoreFileInfo {
    fn from_filename(filename: &str) -> Self {
        let kind = CoreKind::from_filename(filename);
        let minecraft_version = extract_minecraft_version(filename);
        let core_version = extract_version_tokens(filename)
            .into_iter()
            .rev()
            .find(|version| Some(version) != minecraft_version.as_ref());

        Self {
            kind,
            core_version,
            minecraft_version,
            main_class: None,
        }
    }
}

/// 检查服务端核心文件名，无需从磁盘读取。
pub fn inspect_core_filename(filename: &str) -> CoreFileInfo {
    let filename = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(filename);
    CoreFileInfo::from_filename(filename)
}

/// 读取 JAR 清单并将其与基于文件名的服务端核心检测进行协调。
pub fn inspect_core_file(path: &Path) -> Result<CoreFileInfo, CoreParseError> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CoreParseError::InvalidPath { path: path.to_path_buf() })?;
    let mut info = CoreFileInfo::from_filename(filename);

    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
    {
        return Ok(info);
    }

    let manifest = read_manifest(path)?;
    let Some(manifest) = manifest else {
        return Ok(info);
    };

    info.main_class = manifest.main_class;
    if let Some(manifest_version) = manifest.implementation_version {
        info.core_version = Some(manifest_version);
    }
    if let Some(main_class) = info.main_class.as_deref() {
        if let Some(main_class_kind) = core_kind_from_main_class(main_class) {
            info.kind = reconcile_core_kind(info.kind, main_class_kind);
        }
    }

    Ok(info)
}

/// 从文本中提取第一个 Minecraft 风格的版本提示（例如 `1.20.1`）。
pub fn extract_minecraft_version(input: &str) -> Option<String> {
    extract_version_tokens(input)
        .into_iter()
        .find(|version| version.starts_with("1."))
}

fn reconcile_core_kind(filename_kind: CoreKind, main_class_kind: CoreKind) -> CoreKind {
    match (filename_kind, main_class_kind) {
        // NeoForge 安装程序保留了旧的 Forge 安装程序主类。
        (CoreKind::NeoForge | CoreKind::ArclightNeoForge, CoreKind::Forge) => filename_kind,
        (_, main_class_kind) => main_class_kind,
    }
}

fn core_kind_from_main_class(main_class: &str) -> Option<CoreKind> {
    match main_class {
        value if value.starts_with("net.neoforged.serverstarterjar") => Some(CoreKind::NeoForge),
        "net.minecraft.server.MinecraftServer" | "net.minecraft.bundler.Main" => {
            Some(CoreKind::Vanilla)
        }
        "net.minecraft.client.Main" => Some(CoreKind::Unknown),
        "net.minecraftforge.installer.SimpleInstaller" => Some(CoreKind::Forge),
        "net.fabricmc.installer.Main" | "net.fabricmc.installer.ServerLauncher" => {
            Some(CoreKind::Fabric)
        }
        "io.izzel.arclight.server.Launcher" => Some(CoreKind::ArclightForge),
        "catserver.server.CatServerLaunch" | "foxlaunch.FoxServerLauncher" => {
            Some(CoreKind::CatServer)
        }
        "org.bukkit.craftbukkit.Main" | "org.bukkit.craftbukkit.bootstrap.Main" => {
            Some(CoreKind::Bukkit)
        }
        "io.papermc.paperclip.Main" | "com.destroystokyo.paperclip.Paperclip" => {
            Some(CoreKind::Paper)
        }
        "org.leavesmc.leavesclip.Main" => Some(CoreKind::Leaves),
        "net.md_5.bungee.Bootstrap" => Some(CoreKind::Lightfall),
        "com.mohistmc.MohistMCStart" | "com.mohistmc.MohistMC" => Some(CoreKind::Mohist),
        "com.velocitypowered.proxy.Velocity" => Some(CoreKind::Velocity),
        _ => None,
    }
}

struct JarManifest {
    main_class: Option<String>,
    implementation_version: Option<String>,
}

fn read_manifest(path: &Path) -> Result<Option<JarManifest>, CoreParseError> {
    let file = File::open(path)
        .map_err(|source| CoreParseError::Open { path: path.to_path_buf(), source })?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|source| CoreParseError::Archive { path: path.to_path_buf(), source })?;
    let mut manifest = match archive.by_name("META-INF/MANIFEST.MF") {
        Ok(manifest) => manifest,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(source) => {
            return Err(CoreParseError::Manifest { path: path.to_path_buf(), source });
        }
    };

    let mut bytes = Vec::new();
    manifest
        .read_to_end(&mut bytes)
        .map_err(|source| CoreParseError::ManifestRead { path: path.to_path_buf(), source })?;
    Ok(Some(parse_manifest(&String::from_utf8_lossy(&bytes))))
}

fn parse_manifest(content: &str) -> JarManifest {
    let mut entries = Vec::new();
    let mut current_key = String::new();
    let mut current_value = String::new();

    for line in content.lines() {
        if line.is_empty() {
            flush_manifest_entry(&mut entries, &mut current_key, &mut current_value);
            continue;
        }
        if line.starts_with(' ') {
            current_value.push_str(line.trim_start());
            continue;
        }

        flush_manifest_entry(&mut entries, &mut current_key, &mut current_value);
        if let Some((key, value)) = line.split_once(':') {
            current_key.push_str(key.trim());
            current_value.push_str(value.trim());
        }
    }
    flush_manifest_entry(&mut entries, &mut current_key, &mut current_value);

    JarManifest {
        main_class: manifest_value(&entries, "Main-Class"),
        implementation_version: manifest_value(&entries, "Implementation-Version"),
    }
}

fn flush_manifest_entry(
    entries: &mut Vec<(String, String)>,
    current_key: &mut String,
    current_value: &mut String,
) {
    if !current_key.is_empty() {
        entries.push((std::mem::take(current_key), std::mem::take(current_value)));
    }
}

fn manifest_value(entries: &[(String, String)], key: &str) -> Option<String> {
    entries
        .iter()
        .find(|(entry_key, _)| entry_key.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn extract_version_tokens(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() || (index > 0 && bytes[index - 1].is_ascii_digit()) {
            index += 1;
            continue;
        }

        let started_at = index;
        index += 1;
        while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
            index += 1;
        }
        let token = input[started_at..index].trim_end_matches('.');
        if !token.is_empty() {
            tokens.push(token.to_string());
        }
    }

    tokens
}

/// 描述打开或解析服务端核心文件时的失败。
#[derive(Debug)]
pub enum CoreParseError {
    InvalidPath { path: PathBuf },
    Open { path: PathBuf, source: io::Error },
    Archive { path: PathBuf, source: ZipError },
    Manifest { path: PathBuf, source: ZipError },
    ManifestRead { path: PathBuf, source: io::Error },
}

impl fmt::Display for CoreParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path } => {
                write!(formatter, "core file path has no filename: {}", path.display())
            }
            Self::Open { path, source } => {
                write!(formatter, "could not open core file {}: {source}", path.display())
            }
            Self::Archive { path, source } => {
                write!(formatter, "could not parse JAR archive {}: {source}", path.display())
            }
            Self::Manifest { path, source } => {
                write!(formatter, "could not read JAR manifest from {}: {source}", path.display())
            }
            Self::ManifestRead { path, source } => write!(
                formatter,
                "could not read JAR manifest content from {}: {source}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for CoreParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPath { .. } => None,
            Self::Open { source, .. } | Self::ManifestRead { source, .. } => Some(source),
            Self::Archive { source, .. } | Self::Manifest { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zip::write::FileOptions;

    use super::{
        extract_minecraft_version, inspect_core_file, inspect_core_filename, parse_manifest,
        CoreKind,
    };

    fn write_test_jar(filename: &str, manifest: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sealantern-core-provisioning-{}-{}-{}.jar",
            std::process::id(),
            timestamp,
            filename
        ));
        let file = File::create(&path).expect("create test JAR");
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("META-INF/MANIFEST.MF", FileOptions::<()>::default())
            .expect("create manifest entry");
        archive
            .write_all(manifest.as_bytes())
            .expect("write manifest content");
        archive.finish().expect("finish test JAR");
        path
    }

    #[test]
    fn filename_detection_prefers_neoforge_before_forge() {
        let parsed = inspect_core_filename("neoforge-1.20.6-20.6.119.jar");

        assert_eq!(parsed.kind, CoreKind::NeoForge);
        assert_eq!(parsed.minecraft_version.as_deref(), Some("1.20.6"));
        assert_eq!(parsed.core_version.as_deref(), Some("20.6.119"));
    }

    #[test]
    fn manifest_parser_handles_continuation_lines() {
        let manifest = parse_manifest(
            "Manifest-Version: 1.0\r\nMain-Class: net.neoforged.server\r\n starterjar.Main\r\nImplementation-Version: 20.6.119\r\n\r\n",
        );

        assert_eq!(manifest.main_class.as_deref(), Some("net.neoforged.serverstarterjar.Main"));
        assert_eq!(manifest.implementation_version.as_deref(), Some("20.6.119"));
    }

    #[test]
    fn minecraft_version_extraction_ignores_non_minecraft_versions() {
        assert_eq!(extract_minecraft_version("forge-47.2.0.jar"), None);
        assert_eq!(extract_minecraft_version("paper-1.21.4-123.jar"), Some("1.21.4".to_string()));
    }

    #[test]
    fn jar_manifest_keeps_neoforge_filename_over_legacy_forge_installer_class() {
        let path = write_test_jar(
            "neoforge-1.20.6-20.6.119-installer.jar",
            "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\nImplementation-Version: 20.6.119\r\n\r\n",
        );

        let parsed = inspect_core_file(&path).expect("inspect test JAR");
        std::fs::remove_file(&path).expect("remove test JAR");

        assert_eq!(parsed.kind, CoreKind::NeoForge);
        assert_eq!(
            parsed.main_class.as_deref(),
            Some("net.minecraftforge.installer.SimpleInstaller")
        );
        assert_eq!(parsed.core_version.as_deref(), Some("20.6.119"));
    }
}
