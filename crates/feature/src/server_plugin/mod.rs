//! 服务器插件（`plugins` 目录）管理。
//!
//! 插件的权威数据源是 MC 服务器文件系统本身：SeaLantern 只读取 `plugins/`
//! 目录内容与 jar 内的 `plugin.yml` / `bungee.yml` 元数据，并按需重命名、删除
//! 或写入 jar 文件，既不解析也不执行插件实现。
//!
//! 启用与禁用通过文件名后缀表达（`X.jar` ↔ `X.jar.disabled`），与 MC 服务端
//! 的加载规则一致。

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use sealantern_contract::server_plugin::{PluginConfigFile, PluginSummary};
use sealantern_infra::archive::read_zip_entry_text;
use sealantern_infra::fs::SafeRelativePath;

/// 插件所在目录名。
const PLUGINS_DIR: &str = "plugins";
/// jar 内描述文件候选名，按顺序尝试。
const PLUGIN_DESCRIPTORS: [&str; 2] = ["plugin.yml", "bungee.yml"];
/// 描述文件读取上限，超出即视为不可信。
const MAX_DESCRIPTOR_BYTES: usize = 1024 * 1024;
/// 单个配置文件读取上限，避免把巨型文件整体读入内存。
const MAX_CONFIG_FILE_BYTES: usize = 4 * 1024 * 1024;
/// 会被读取的配置文件扩展名。
const CONFIG_FILE_EXTENSIONS: [&str; 4] = ["yml", "yaml", "json", "properties"];
/// 禁用状态的文件名后缀。
const DISABLED_SUFFIX: &str = ".disabled";

/// 服务器插件管理错误。
#[derive(Debug, thiserror::Error)]
pub enum ServerPluginError {
    /// 文件名不合法：为空、含路径分量，或命中 Windows 歧义名。
    #[error("非法的插件文件名: {0}")]
    InvalidFileName(String),

    /// 目标插件文件不存在。
    #[error("插件不存在: {0}")]
    NotFound(String),

    /// 文件系统操作失败。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 归档读取失败。
    #[error("归档错误: {0}")]
    Archive(#[from] sealantern_infra::archive::ArchiveError),
}

/// jar 内描述文件的解析结果。
///
/// 全部字段可选：不同服务端实现（Bukkit / BungeeCord / 混合端）填写的字段并不一致，
/// 缺失的字段由上层回落到默认值，不视为错误。
#[derive(Debug, Default, Deserialize)]
struct PluginDescriptor {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    authors: Option<Vec<String>>,
    main: Option<String>,
}

/// `plugins` 目录管理器。
pub struct ServerPluginManager {
    plugins_dir: PathBuf,
}

impl ServerPluginManager {
    /// 以服务器目录构造；调用时不会创建任何目录。
    pub fn new(server_path: impl AsRef<Path>) -> Self {
        Self {
            plugins_dir: server_path.as_ref().join(PLUGINS_DIR),
        }
    }

    /// 列出全部插件，包含已禁用（`.jar.disabled`）的文件。
    ///
    /// 目录不存在时返回空列表；单个 jar 无法读取或解析时按元数据缺失处理，
    /// 仍然出现在结果中，不牵连其它插件。
    pub fn list(&self) -> Result<Vec<PluginSummary>, ServerPluginError> {
        if !self.plugins_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();
        for entry in fs::read_dir(&self.plugins_dir)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let enabled = !file_name.ends_with(DISABLED_SUFFIX);
            let base_name = file_name.strip_suffix(DISABLED_SUFFIX).unwrap_or(file_name);
            if !base_name.ends_with(".jar") {
                continue;
            }

            plugins.push(self.summarize(&path, base_name, enabled)?);
        }

        plugins.sort_by(|left, right| left.file_name.cmp(&right.file_name));
        Ok(plugins)
    }

    /// 读取某个插件配置目录下的文本文件。
    ///
    /// `plugin_name` 由调用方给出，因此按单个路径分量校验；目录不存在时返回空列表。
    pub fn read_config_files(
        &self,
        plugin_name: &str,
    ) -> Result<Vec<PluginConfigFile>, ServerPluginError> {
        let folder_name = validate_component(plugin_name)?;
        let folder = self.plugins_dir.join(folder_name);
        if !folder.is_dir() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        collect_config_files(&folder, &mut files)?;
        files.sort_by(|left, right| left.file_path.cmp(&right.file_path));
        Ok(files)
    }

    /// 启用或禁用插件，通过重命名在 `.jar` 与 `.jar.disabled` 之间切换。
    pub fn set_enabled(&self, file_name: &str, enabled: bool) -> Result<(), ServerPluginError> {
        let base_name = normalize_jar_file_name(file_name)?;
        let enabled_path = self.plugins_dir.join(&base_name);
        let disabled_path = self
            .plugins_dir
            .join(format!("{base_name}{DISABLED_SUFFIX}"));

        // 目标是启用态时，源文件当前应是禁用名，反之亦然。
        let (source, target) = if enabled {
            (&disabled_path, &enabled_path)
        } else {
            (&enabled_path, &disabled_path)
        };

        if !source.is_file() {
            return Err(ServerPluginError::NotFound(base_name));
        }
        fs::rename(source, target)?;
        Ok(())
    }

    /// 永久删除插件，同时清理启用与禁用两种文件名。
    pub fn delete(&self, file_name: &str) -> Result<(), ServerPluginError> {
        let base_name = normalize_jar_file_name(file_name)?;
        let enabled_path = self.plugins_dir.join(&base_name);
        let disabled_path = self
            .plugins_dir
            .join(format!("{base_name}{DISABLED_SUFFIX}"));

        let mut removed = false;
        for path in [&enabled_path, &disabled_path] {
            if path.is_file() {
                fs::remove_file(path)?;
                removed = true;
            }
        }

        if !removed {
            return Err(ServerPluginError::NotFound(base_name));
        }
        Ok(())
    }

    /// 安装插件：把字节内容写入 `plugins/<file_name>`，同名文件会被覆盖。
    pub fn install(&self, file_name: &str, data: &[u8]) -> Result<(), ServerPluginError> {
        let base_name = normalize_jar_file_name(file_name)?;
        fs::create_dir_all(&self.plugins_dir)?;
        fs::write(self.plugins_dir.join(base_name), data)?;
        Ok(())
    }

    /// 汇总单个 jar 的元数据；描述文件缺失或语法错误时回落到默认值。
    fn summarize(
        &self,
        jar_path: &Path,
        base_name: &str,
        enabled: bool,
    ) -> Result<PluginSummary, ServerPluginError> {
        let descriptor = Self::read_descriptor(jar_path).unwrap_or_default();

        let file_stem = base_name.strip_suffix(".jar").unwrap_or(base_name);
        let name = descriptor.name.filter(|value| !value.is_empty());
        let version = descriptor.version.filter(|value| !value.is_empty());

        // 配置目录按插件声明的名称查找，缺失时回落到 jar 文件名。
        let config_folder = name.clone().unwrap_or_else(|| file_stem.to_owned());
        let has_config_folder = self.plugins_dir.join(&config_folder).is_dir();

        let author = descriptor
            .author
            .or_else(|| {
                descriptor
                    .authors
                    .and_then(|authors| authors.into_iter().next())
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "未知".to_owned());

        let display_name = name.unwrap_or_else(|| file_stem.to_owned());
        let display_version = version.unwrap_or_else(|| "未知".to_owned());

        Ok(PluginSummary {
            id: format!("{display_name}-{display_version}"),
            name: display_name,
            version: display_version,
            description: descriptor.description.unwrap_or_default(),
            author,
            file_name: base_name.to_owned(),
            file_size: jar_path.metadata().map(|meta| meta.len()).unwrap_or(0),
            enabled,
            main_class: descriptor.main.unwrap_or_default(),
            has_config_folder,
            config_files: Vec::new(),
        })
    }

    /// 从 jar 中读取并解析描述文件；任一步失败都返回 `None`。
    fn read_descriptor(jar_path: &Path) -> Option<PluginDescriptor> {
        for candidate in PLUGIN_DESCRIPTORS {
            match read_zip_entry_text(jar_path, candidate, MAX_DESCRIPTOR_BYTES) {
                // 描述文件存在但语法错误时不再尝试其它候选名：这是插件自身的问题。
                Ok(Some(content)) => return serde_yaml::from_str(&content).ok(),
                Ok(None) => continue,
                // jar 损坏或读取超限：该插件按元数据缺失处理。
                Err(_) => return None,
            }
        }
        None
    }
}

/// 校验调用方给出的名称是单个可移植路径分量。
fn validate_component(name: &str) -> Result<&str, ServerPluginError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.contains('/') {
        return Err(ServerPluginError::InvalidFileName(name.to_owned()));
    }
    SafeRelativePath::parse(trimmed)
        .map_err(|_| ServerPluginError::InvalidFileName(name.to_owned()))?;
    Ok(trimmed)
}

/// 把调用方给出的文件名规范化为不带 `.disabled` 后缀的 jar 文件名。
fn normalize_jar_file_name(file_name: &str) -> Result<String, ServerPluginError> {
    let trimmed = validate_component(file_name)?;
    let base_name = trimmed.strip_suffix(DISABLED_SUFFIX).unwrap_or(trimmed);
    if !base_name.ends_with(".jar") {
        return Err(ServerPluginError::InvalidFileName(file_name.to_owned()));
    }
    Ok(base_name.to_owned())
}

/// 递归收集配置目录下的文本文件，跳过符号链接以免越过目录边界。
fn collect_config_files(
    directory: &Path,
    files: &mut Vec<PluginConfigFile>,
) -> Result<(), ServerPluginError> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_config_files(&path, files)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        let file_type = extension.to_ascii_lowercase();
        if !CONFIG_FILE_EXTENSIONS.contains(&file_type.as_str()) {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        // 读取失败（含超限）只跳过该文件，不影响同目录的其它配置。
        let Some(content) = read_text_bounded(&path, MAX_CONFIG_FILE_BYTES) else {
            continue;
        };

        files.push(PluginConfigFile {
            file_name: file_name.to_owned(),
            content,
            file_type,
            file_path: path.to_string_lossy().into_owned(),
        });
    }
    Ok(())
}

/// 有界读取文本文件，超出上限或编码非法时返回 `None`。
///
/// `sealantern_infra::fs::read_string_limited` 是异步接口，而本模块整体在阻塞
/// 线程上执行，这里用同步方式实现同样的「先按上限截断再读」语义。
fn read_text_bounded(path: &Path, max_bytes: usize) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut content = String::new();
    file.take(max_bytes as u64 + 1)
        .read_to_string(&mut content)
        .ok()?;
    if content.len() > max_bytes {
        return None;
    }
    Some(content)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    /// 在给定目录下生成一个内含 `plugin.yml` 的测试 jar。
    fn write_plugin_jar(directory: &Path, file_name: &str, descriptor: &str) {
        fs::create_dir_all(directory).unwrap();
        let file = fs::File::create(directory.join(file_name)).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("plugin.yml", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(descriptor.as_bytes()).unwrap();
        writer.finish().unwrap();
    }

    fn plugin_manager(server: &Path) -> ServerPluginManager {
        ServerPluginManager::new(server)
    }

    #[test]
    fn lists_enabled_and_disabled_plugins_with_metadata() {
        let server = tempdir().unwrap();
        let plugins_dir = server.path().join(PLUGINS_DIR);

        write_plugin_jar(
            &plugins_dir,
            "EssentialsX.jar",
            "name: EssentialsX\nversion: 2.20.1\nmain: com.earth2me.essentials.Essentials\nauthor: Zenexer\ndescription: 基础指令集\n",
        );
        write_plugin_jar(&plugins_dir, "Legacy.jar.disabled", "name: Legacy\nversion: 1.0\n");

        let plugins = plugin_manager(server.path()).list().unwrap();
        assert_eq!(plugins.len(), 2);

        let essentials = plugins
            .iter()
            .find(|item| item.name == "EssentialsX")
            .expect("含 EssentialsX");
        assert_eq!(essentials.file_name, "EssentialsX.jar");
        assert_eq!(essentials.version, "2.20.1");
        assert_eq!(essentials.id, "EssentialsX-2.20.1");
        assert_eq!(essentials.main_class, "com.earth2me.essentials.Essentials");
        assert_eq!(essentials.author, "Zenexer");
        assert!(essentials.enabled);
        assert!(essentials.file_size > 0);

        let legacy = plugins
            .iter()
            .find(|item| item.name == "Legacy")
            .expect("含 Legacy");
        assert_eq!(legacy.file_name, "Legacy.jar", "禁用插件的文件名不应带后缀");
        assert!(!legacy.enabled);
    }

    #[test]
    fn keeps_plugins_without_a_readable_descriptor_using_fallbacks() {
        let server = tempdir().unwrap();
        let plugins_dir = server.path().join(PLUGINS_DIR);
        fs::create_dir_all(&plugins_dir).unwrap();
        // 不是合法 zip，读取元数据必然失败。
        fs::write(plugins_dir.join("Broken.jar"), b"not a zip").unwrap();

        let plugins = plugin_manager(server.path()).list().unwrap();
        assert_eq!(plugins.len(), 1, "无法解析的插件仍应出现在列表中");
        assert_eq!(plugins[0].name, "Broken", "名称应回落到文件名");
        assert_eq!(plugins[0].version, "未知");
        assert_eq!(plugins[0].author, "未知");
        assert!(plugins[0].main_class.is_empty());
    }

    #[test]
    fn lists_a_config_folder_only_when_it_matches_the_declared_name() {
        let server = tempdir().unwrap();
        let plugins_dir = server.path().join(PLUGINS_DIR);
        write_plugin_jar(&plugins_dir, "EssentialsX.jar", "name: EssentialsX\n");
        fs::create_dir_all(plugins_dir.join("EssentialsX")).unwrap();

        let plugins = plugin_manager(server.path()).list().unwrap();
        assert!(plugins[0].has_config_folder);
        assert!(plugins[0].config_files.is_empty(), "列表接口不应读取配置内容");
    }

    #[test]
    fn returns_empty_list_when_plugins_directory_is_absent() {
        let server = tempdir().unwrap();
        assert!(plugin_manager(server.path()).list().unwrap().is_empty());
        // 读操作不创建目录。
        assert!(!server.path().join(PLUGINS_DIR).exists());
    }

    #[test]
    fn toggles_plugin_by_renaming() {
        let server = tempdir().unwrap();
        let plugins_dir = server.path().join(PLUGINS_DIR);
        write_plugin_jar(&plugins_dir, "EssentialsX.jar", "name: EssentialsX\n");

        plugin_manager(server.path())
            .set_enabled("EssentialsX.jar", false)
            .unwrap();
        assert!(plugins_dir.join("EssentialsX.jar.disabled").is_file());
        assert!(!plugins_dir.join("EssentialsX.jar").exists());

        // 传入带禁用后缀的名字也应被接受。
        plugin_manager(server.path())
            .set_enabled("EssentialsX.jar.disabled", true)
            .unwrap();
        assert!(plugins_dir.join("EssentialsX.jar").is_file());
        assert!(!plugins_dir.join("EssentialsX.jar.disabled").exists());
    }

    #[test]
    fn deleting_removes_both_enabled_and_disabled_files() {
        let server = tempdir().unwrap();
        let plugins_dir = server.path().join(PLUGINS_DIR);
        write_plugin_jar(&plugins_dir, "EssentialsX.jar", "name: EssentialsX\n");
        fs::write(plugins_dir.join("EssentialsX.jar.disabled"), b"stale").unwrap();

        plugin_manager(server.path())
            .delete("EssentialsX.jar")
            .unwrap();
        assert!(!plugins_dir.join("EssentialsX.jar").exists());
        assert!(!plugins_dir.join("EssentialsX.jar.disabled").exists());
    }

    #[test]
    fn installs_plugin_bytes_and_creates_directory() {
        let server = tempdir().unwrap();
        plugin_manager(server.path())
            .install("New.jar", b"jar bytes")
            .unwrap();
        assert_eq!(
            fs::read(server.path().join(PLUGINS_DIR).join("New.jar")).unwrap(),
            b"jar bytes"
        );
    }

    #[test]
    fn reads_only_whitelisted_config_files_recursively() {
        let server = tempdir().unwrap();
        let config_dir = server.path().join(PLUGINS_DIR).join("EssentialsX");
        fs::create_dir_all(config_dir.join("nested")).unwrap();
        fs::write(config_dir.join("config.yml"), "motd: hello").unwrap();
        fs::write(config_dir.join("nested").join("extra.json"), "{}").unwrap();
        fs::write(config_dir.join("notes.txt"), "ignored").unwrap();

        let files = plugin_manager(server.path())
            .read_config_files("EssentialsX")
            .unwrap();
        assert_eq!(files.len(), 2, "白名单外的文件不应被读取");
        assert!(
            files
                .iter()
                .any(|item| item.file_name == "config.yml" && item.file_type == "yml")
        );
        assert!(files.iter().any(|item| item.file_name == "extra.json"));
    }

    /// 路径穿越是本模块最需要防住的输入：所有文件名都直接来自客户端。
    #[test]
    fn rejects_file_names_that_escape_the_plugins_directory() {
        let server = tempdir().unwrap();
        let manager = plugin_manager(server.path());

        for name in [
            "../escape.jar",
            "..\\escape.jar",
            "nested/EssentialsX.jar",
            "/absolute.jar",
            "CON.jar",
            "",
            "   ",
            "not-a-jar.txt",
        ] {
            assert!(
                matches!(manager.delete(name), Err(ServerPluginError::InvalidFileName(_))),
                "delete 应拒绝 {name}"
            );
            assert!(
                matches!(
                    manager.set_enabled(name, false),
                    Err(ServerPluginError::InvalidFileName(_))
                ),
                "set_enabled 应拒绝 {name}"
            );
            assert!(
                matches!(manager.install(name, b"x"), Err(ServerPluginError::InvalidFileName(_))),
                "install 应拒绝 {name}"
            );
        }

        for name in ["../secret", "nested/name", ""] {
            assert!(
                matches!(
                    manager.read_config_files(name),
                    Err(ServerPluginError::InvalidFileName(_))
                ),
                "read_config_files 应拒绝 {name}"
            );
        }
    }

    #[test]
    fn reports_missing_plugins_instead_of_silently_succeeding() {
        let server = tempdir().unwrap();
        let manager = plugin_manager(server.path());

        assert!(matches!(
            manager.set_enabled("Absent.jar", false),
            Err(ServerPluginError::NotFound(_))
        ));
        assert!(matches!(manager.delete("Absent.jar"), Err(ServerPluginError::NotFound(_))));
    }
}
