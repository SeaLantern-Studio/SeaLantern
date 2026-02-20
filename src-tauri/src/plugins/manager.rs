use crate::models::plugin::{
    MissingDependency, PluginDependency, PluginInfo, PluginInstallResult, PluginState,
};
use crate::plugins::api::{
    emit_log_event, emit_ui_event, new_api_registry, ApiRegistry, ApiRegistryOps,
};
use crate::plugins::loader::PluginLoader;
use crate::plugins::runtime::{kill_all_processes, PluginRuntime};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub type SharedRuntimes = Arc<RwLock<HashMap<String, PluginRuntime>>>;

pub fn new_shared_runtimes() -> SharedRuntimes {
    Arc::new(RwLock::new(HashMap::new()))
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,

    runtimes: SharedRuntimes,

    plugins_dir: PathBuf,

    data_dir: PathBuf,

    api_registry: ApiRegistry,
}

impl PluginManager {
    pub fn new(plugins_dir: PathBuf, data_dir: PathBuf) -> Self {
        if let Err(e) = fs::create_dir_all(&plugins_dir) {
            eprintln!("[ERROR] Failed to create plugins directory: {}", e);
        }
        if let Err(e) = fs::create_dir_all(&data_dir) {
            eprintln!("[ERROR] Failed to create data directory: {}", e);
        }

        Self {
            plugins: HashMap::new(),
            runtimes: new_shared_runtimes(),
            plugins_dir,
            data_dir,
            api_registry: new_api_registry(),
        }
    }

    pub fn get_shared_runtimes(&self) -> SharedRuntimes {
        Arc::clone(&self.runtimes)
    }

    pub fn get_api_registry(&self) -> ApiRegistry {
        Arc::clone(&self.api_registry)
    }

    pub fn scan_plugins(&mut self) -> Result<Vec<PluginInfo>, String> {
        {
            let mut runtimes = self.runtimes.write().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            for (id, runtime) in runtimes.drain() {
                kill_all_processes(&runtime.process_registry);
                if let Err(e) = runtime.call_lifecycle("onDisable") {
                    eprintln!(
                        "[WARN] Failed to call onDisable for plugin '{}' during rescan: {}",
                        id, e
                    );
                }
                if let Err(e) = runtime.call_lifecycle("onUnload") {
                    eprintln!(
                        "[WARN] Failed to call onUnload for plugin '{}' during rescan: {}",
                        id, e
                    );
                }
            }
        }

        self.plugins.clear();

        let plugin_dirs = PluginLoader::discover_plugins(&self.plugins_dir)?;

        for plugin_dir in &plugin_dirs {
            match PluginLoader::load_manifest(plugin_dir) {
                Ok(manifest) => {
                    let state = match PluginLoader::validate_manifest(&manifest) {
                        Ok(()) => PluginState::Loaded,
                        Err(e) => {
                            eprintln!("Invalid manifest in {}: {}", plugin_dir.display(), e);
                            PluginState::Error(e)
                        }
                    };

                    let plugin_info = PluginInfo {
                        manifest: manifest.clone(),
                        state,
                        path: plugin_dir.to_string_lossy().to_string(),
                        missing_dependencies: Vec::new(),
                    };

                    self.plugins.insert(manifest.id.clone(), plugin_info);
                }
                Err(e) => {
                    eprintln!("Failed to load manifest from {}: {}", plugin_dir.display(), e);
                }
            }
        }

        self.update_all_missing_dependencies();

        Ok(self.plugins.values().cloned().collect())
    }

    fn update_all_missing_dependencies(&mut self) {
        let plugin_manifests: Vec<(String, crate::models::plugin::PluginManifest)> = self
            .plugins
            .iter()
            .map(|(id, info)| (id.clone(), info.manifest.clone()))
            .collect();

        for (plugin_id, manifest) in plugin_manifests {
            let missing = self.get_missing_dependencies(&manifest);
            if let Some(info) = self.plugins.get_mut(&plugin_id) {
                info.missing_dependencies = missing;
            }
        }
    }

    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin_info = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?
            .clone();

        if matches!(plugin_info.state, PluginState::Enabled) {
            return Ok(());
        }

        let missing_deps = self.check_dependencies(&plugin_info.manifest.dependencies);
        if !missing_deps.is_empty() {
            return Err(format!(
                "无法启用插件 '{}'：缺少必须依赖：{}",
                plugin_info.manifest.name,
                missing_deps.join(", ")
            ));
        }

        let missing_optional = self.check_dependencies(&plugin_info.manifest.optional_dependencies);
        if !missing_optional.is_empty() {
            eprintln!(
                "[插件] '{}' 的可选依赖未满足：{}（部分功能可能受限）",
                plugin_info.manifest.name,
                missing_optional.join(", ")
            );
        }

        let plugin_dir = PathBuf::from(&plugin_info.path);
        let plugin_data_dir = self.data_dir.join(plugin_id);

        if !plugin_info.manifest.include.is_empty() {
            Self::copy_included_resources(
                &plugin_dir,
                &plugin_data_dir,
                &plugin_info.manifest.include,
            )?;
        }

        let permissions = plugin_info.manifest.permissions.clone();

        let runtime = PluginRuntime::new(
            plugin_id,
            &plugin_dir,
            &plugin_data_dir,
            Arc::clone(&self.api_registry),
            permissions,
        )?;

        let main_file = plugin_dir.join(&plugin_info.manifest.main);
        runtime.load_file(&main_file)?;

        runtime.call_lifecycle("onLoad")?;

        {
            let mut runtimes = self.runtimes.write().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            runtimes.insert(plugin_id.to_string(), runtime);
        }

        let enable_result = {
            let runtimes = self.runtimes.read().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });

            if let Some(r) = runtimes.get(plugin_id) {
                r.call_lifecycle("onEnable")
            } else {
                Err("Runtime not found after insertion".to_string())
            }
        };

        if let Err(e) = enable_result {
            let error_msg = format!("Failed to call onEnable: {}", e);
            let _ = emit_log_event(plugin_id, "error", &error_msg);

            {
                let mut runtimes = self.runtimes.write().unwrap_or_else(|e| {
                    eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                    e.into_inner()
                });
                if let Some(runtime) = runtimes.get_mut(plugin_id) {
                    let _ = runtime.call_lifecycle("onDisable");
                    let _ = runtime.call_lifecycle("onUnload");
                    runtime.cleanup();
                    kill_all_processes(&runtime.process_registry);
                }
                runtimes.remove(plugin_id);
            }
            return Err(format!("Failed to enable plugin: {}", e));
        }

        if let Some(info) = self.plugins.get_mut(plugin_id) {
            info.state = PluginState::Enabled;
        }

        self.update_all_missing_dependencies();

        self.save_enabled_plugins();

        Ok(())
    }

    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<Vec<String>, String> {
        let mut visited = HashSet::new();
        self.disable_plugin_internal(plugin_id, &mut visited)
    }

    fn disable_plugin_internal(
        &mut self,
        plugin_id: &str,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<String>, String> {
        if visited.contains(plugin_id) {
            return Ok(Vec::new());
        }
        visited.insert(plugin_id.to_string());

        let mut disabled_plugins = Vec::new();

        let plugin_info = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        if !matches!(plugin_info.state, PluginState::Enabled) {
            return Ok(disabled_plugins);
        }

        let dependent_ids = self.get_dependent_plugin_ids(plugin_id);

        for dep_id in dependent_ids {
            disabled_plugins.push(dep_id.clone());

            if let Ok(mut cascaded) = self.disable_plugin_internal(&dep_id, visited) {
                disabled_plugins.append(&mut cascaded);
            }
        }

        {
            let runtimes = self.runtimes.read().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            if let Some(runtime) = runtimes.get(plugin_id) {
                if let Err(e) = runtime.call_lifecycle("onDisable") {
                    eprintln!("Error calling onDisable for '{}': {}", plugin_id, e);

                    let error_msg = format!("Failed to call onDisable: {}", e);
                    let _ = emit_log_event(plugin_id, "error", &error_msg);
                }
            }
        }

        if let Err(e) = emit_ui_event(plugin_id, "remove_all", "", "") {
            eprintln!("[WARN] Failed to emit remove_all UI event for '{}': {}", plugin_id, e);
        }

        crate::plugins::api::clear_plugin_sidebar_snapshot(plugin_id);
        crate::plugins::api::clear_plugin_component_snapshot(plugin_id);
        crate::plugins::api::clear_plugin_context_menu_snapshot(plugin_id);

        self.api_registry.clear_plugin_apis(plugin_id);

        {
            let runtimes = self.runtimes.read().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            if let Some(runtime) = runtimes.get(plugin_id) {
                runtime.cleanup();

                kill_all_processes(&runtime.process_registry);
            }
        }

        {
            let mut runtimes = self.runtimes.write().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            runtimes.remove(plugin_id);
        }

        if let Some(info) = self.plugins.get_mut(plugin_id) {
            info.state = PluginState::Disabled;
        }

        self.save_enabled_plugins();

        Ok(disabled_plugins)
    }

    fn check_dependencies(&self, dependencies: &[PluginDependency]) -> Vec<String> {
        let mut missing = Vec::new();
        for dep in dependencies {
            let dep_id = dep.id();
            match self.plugins.get(dep_id) {
                Some(dep_info) => {
                    if !matches!(dep_info.state, PluginState::Enabled) {
                        missing.push(format!("{} (未启用)", dep_id));
                    } else if !dep.is_satisfied_by(&dep_info.manifest.version) {
                        let req = dep.version_requirement().unwrap_or("any");
                        missing.push(format!(
                            "{} (版本 {} 不满足要求 {})",
                            dep_id, dep_info.manifest.version, req
                        ));
                    }
                }
                None => {
                    if let Some(req) = dep.version_requirement() {
                        missing.push(format!("{} {} (未安装)", dep_id, req));
                    } else {
                        missing.push(format!("{} (未安装)", dep_id));
                    }
                }
            }
        }
        missing
    }

    fn get_dependent_plugin_ids(&self, plugin_id: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        for (id, info) in &self.plugins {
            if !matches!(info.state, PluginState::Enabled) {
                continue;
            }

            if info
                .manifest
                .dependencies
                .iter()
                .any(|d| d.id() == plugin_id)
            {
                dependents.push(id.clone());
            }
        }
        dependents
    }

    fn save_enabled_plugins(&self) {
        let enabled: Vec<&str> = self
            .plugins
            .iter()
            .filter(|(_, info)| matches!(info.state, PluginState::Enabled))
            .map(|(id, _)| id.as_str())
            .collect();
        let path = self.data_dir.join("enabled_plugins.json");
        match serde_json::to_string(&enabled) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("[WARN] Failed to save enabled plugins: {}", e);
                }
            }
            Err(e) => eprintln!("[WARN] Failed to serialize enabled plugins: {}", e),
        }
    }

    fn copy_included_resources(
        plugin_dir: &Path,
        data_dir: &Path,
        includes: &[String],
    ) -> Result<(), String> {
        fs::create_dir_all(data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

        for pattern in includes {
            let clean = pattern.trim_end_matches('/');
            let src = plugin_dir.join(clean);
            if !src.exists() {
                eprintln!("[插件] include 资源不存在，跳过: {}", src.display());
                continue;
            }
            let dest = data_dir.join(clean);
            if src.is_dir() {
                Self::copy_dir_recursive(&src, &dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent dir: {}", e))?;
                }
                fs::copy(&src, &dest)
                    .map_err(|e| format!("Failed to copy {}: {}", src.display(), e))?;
            }
        }
        Ok(())
    }

    fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
        fs::create_dir_all(dest)
            .map_err(|e| format!("Failed to create dir {}: {}", dest.display(), e))?;
        let entries = fs::read_dir(src)
            .map_err(|e| format!("Failed to read dir {}: {}", src.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());
            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dest_path)?;
            } else {
                fs::copy(&src_path, &dest_path)
                    .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
            }
        }
        Ok(())
    }

    fn load_enabled_plugin_ids(&self) -> Vec<String> {
        let path = self.data_dir.join("enabled_plugins.json");
        match fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str::<Vec<String>>(&json).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    pub fn auto_enable_plugins(&mut self) {
        let ids = self.load_enabled_plugin_ids();
        if ids.is_empty() {
            return;
        }

        let mut enabled_set: HashSet<String> = HashSet::new();
        let mut remaining: Vec<String> = ids;
        let mut max_passes = remaining.len() + 1;
        while !remaining.is_empty() && max_passes > 0 {
            max_passes -= 1;
            let mut next = Vec::new();
            for id in remaining {
                let deps_ok = if let Some(info) = self.plugins.get(&id) {
                    info.manifest
                        .dependencies
                        .iter()
                        .all(|d| enabled_set.contains(d.id()))
                } else {
                    false
                };
                if deps_ok {
                    if let Err(e) = self.enable_plugin(&id) {
                        eprintln!("[WARN] Auto-enable plugin '{}' failed: {}", id, e);
                    } else {
                        enabled_set.insert(id);
                    }
                } else {
                    next.push(id);
                }
            }
            remaining = next;
        }

        for id in remaining {
            eprintln!("[WARN] Auto-enable skipped '{}': dependencies not met", id);
        }
    }

    pub fn disable_all_plugins(&mut self) {
        let enabled_ids: Vec<String> = self
            .plugins
            .iter()
            .filter(|(_, info)| matches!(info.state, PluginState::Enabled))
            .map(|(id, _)| id.clone())
            .collect();
        for id in enabled_ids {
            if let Err(e) = self.disable_plugin(&id) {
                eprintln!("[WARN] Failed to disable plugin '{}' on exit: {}", id, e);
            }
        }
    }

    pub fn get_plugin_list(&self) -> Vec<PluginInfo> {
        self.plugins.values().cloned().collect()
    }

    pub fn get_nav_items(&self) -> Vec<serde_json::Value> {
        let mut nav_items = Vec::new();

        for (plugin_id, info) in &self.plugins {
            if !matches!(info.state, PluginState::Enabled) {
                continue;
            }

            if let Some(ref ui) = info.manifest.ui {
                if let Some(ref sidebar) = ui.sidebar {
                    nav_items.push(serde_json::json!({
                        "plugin_id": plugin_id,
                        "group": sidebar.group,
                        "label": sidebar.label,
                        "icon": sidebar.icon,
                        "priority": sidebar.priority.unwrap_or(0),
                        "pages": ui.pages.iter().map(|p| {
                            serde_json::json!({
                                "id": p.id,
                                "title": p.title,
                                "path": p.path,
                                "icon": p.icon,
                            })
                        }).collect::<Vec<_>>(),
                    }));
                }
            }
        }

        nav_items
    }

    pub fn install_plugin(&mut self, path: &Path) -> Result<PluginInstallResult, String> {
        let plugin_info = if path.extension().is_some_and(|ext| ext == "zip") {
            self.install_plugin_from_zip(path)?
        } else if path.file_name().is_some_and(|name| name == "manifest.json") {
            let plugin_dir = path.parent().ok_or("Invalid manifest path")?;
            self.install_plugin_from_dir(plugin_dir)?
        } else if path.is_dir() {
            self.install_plugin_from_dir(path)?
        } else {
            return Err(
                "Unsupported file format. Please provide a .zip file or manifest.json".to_string()
            );
        };

        let missing_dependencies = self.get_missing_dependencies(&plugin_info.manifest);

        Ok(PluginInstallResult {
            plugin: plugin_info,
            missing_dependencies,
            untrusted_url: false,
        })
    }

    fn get_missing_dependencies(
        &self,
        manifest: &crate::models::plugin::PluginManifest,
    ) -> Vec<MissingDependency> {
        let mut missing = Vec::new();

        for dep in &manifest.dependencies {
            let dep_id = dep.id();
            let is_missing = match self.plugins.get(dep_id) {
                Some(dep_info) => {
                    !dep.is_satisfied_by(&dep_info.manifest.version)
                        || !matches!(dep_info.state, crate::models::plugin::PluginState::Enabled)
                }
                None => true,
            };

            if is_missing {
                missing.push(MissingDependency {
                    id: dep_id.to_string(),
                    version_requirement: dep.version_requirement().map(|s| s.to_string()),
                    required: true,
                });
            }
        }

        for dep in &manifest.optional_dependencies {
            let dep_id = dep.id();
            let is_missing = match self.plugins.get(dep_id) {
                Some(dep_info) => {
                    !dep.is_satisfied_by(&dep_info.manifest.version)
                        || !matches!(dep_info.state, crate::models::plugin::PluginState::Enabled)
                }
                None => true,
            };

            if is_missing {
                missing.push(MissingDependency {
                    id: dep_id.to_string(),
                    version_requirement: dep.version_requirement().map(|s| s.to_string()),
                    required: false,
                });
            }
        }

        missing
    }

    pub fn install_plugin_from_dir(&mut self, source_dir: &Path) -> Result<PluginInfo, String> {
        let manifest_path = source_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(format!("manifest.json not found in {}", source_dir.display()));
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

        let manifest: crate::models::plugin::PluginManifest =
            serde_json::from_str(&manifest_content)
                .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

        PluginLoader::validate_manifest(&manifest)?;

        let plugin_id = manifest.id.clone();

        if let Some(existing) = self.plugins.get(&plugin_id) {
            if matches!(existing.state, PluginState::Enabled) {
                return Err(format!(
                    "插件 '{}' 正在运行中，请先禁用后再进行替换",
                    existing.manifest.name
                ));
            }
        }

        let target_dir = self.plugins_dir.join(&plugin_id);

        let source_canonical = source_dir
            .canonicalize()
            .map_err(|e| format!("Failed to resolve source path: {}", e))?;
        let target_canonical = if target_dir.exists() {
            Some(
                target_dir
                    .canonicalize()
                    .map_err(|e| format!("Failed to resolve target path: {}", e))?,
            )
        } else {
            None
        };

        if target_canonical.as_ref() == Some(&source_canonical) {
            let loaded_manifest = PluginLoader::load_manifest(&target_dir)?;
            PluginLoader::validate_manifest(&loaded_manifest)?;

            let missing_deps = self.get_missing_dependencies(&loaded_manifest);

            let plugin_info = PluginInfo {
                manifest: loaded_manifest,
                state: PluginState::Loaded,
                path: target_dir.to_string_lossy().to_string(),
                missing_dependencies: missing_deps,
            };

            self.plugins.insert(plugin_id, plugin_info.clone());
            return Ok(plugin_info);
        }

        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("Failed to remove existing plugin directory: {}", e))?;
        }

        Self::copy_dir_recursive(source_dir, &target_dir)?;

        let loaded_manifest = PluginLoader::load_manifest(&target_dir)?;
        PluginLoader::validate_manifest(&loaded_manifest)?;

        let missing_deps = self.get_missing_dependencies(&loaded_manifest);

        let plugin_info = PluginInfo {
            manifest: loaded_manifest,
            state: PluginState::Loaded,
            path: target_dir.to_string_lossy().to_string(),
            missing_dependencies: missing_deps,
        };

        self.plugins.insert(plugin_id, plugin_info.clone());

        Ok(plugin_info)
    }

    fn install_plugin_from_zip(&mut self, zip_path: &Path) -> Result<PluginInfo, String> {
        let file = File::open(zip_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

        let (manifest_content, prefix) = self.find_manifest_in_zip(&mut archive)?;

        let manifest: crate::models::plugin::PluginManifest =
            serde_json::from_str(&manifest_content)
                .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

        PluginLoader::validate_manifest(&manifest)?;

        let plugin_id = manifest.id.clone();

        if let Some(existing) = self.plugins.get(&plugin_id) {
            if matches!(existing.state, PluginState::Enabled) {
                return Err(format!(
                    "插件 '{}' 正在运行中，请先禁用后再进行替换",
                    existing.manifest.name
                ));
            }
        }

        let target_dir = self.plugins_dir.join(&plugin_id);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("Failed to remove existing plugin directory: {}", e))?;
        }
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

        self.extract_zip_to_dir(&mut archive, &prefix, &target_dir)?;

        let loaded_manifest = PluginLoader::load_manifest(&target_dir)?;
        PluginLoader::validate_manifest(&loaded_manifest)?;

        let missing_deps = self.get_missing_dependencies(&loaded_manifest);

        let plugin_info = PluginInfo {
            manifest: loaded_manifest,
            state: PluginState::Loaded,
            path: target_dir.to_string_lossy().to_string(),
            missing_dependencies: missing_deps,
        };

        self.plugins.insert(plugin_id, plugin_info.clone());

        Ok(plugin_info)
    }

    fn find_manifest_in_zip(
        &self,
        archive: &mut zip::ZipArchive<File>,
    ) -> Result<(String, String), String> {
        if let Ok(mut file) = archive.by_name("manifest.json") {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read manifest.json: {}", e))?;
            return Ok((content, String::new()));
        }

        let mut found_prefix: Option<String> = None;
        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;
            let name = file.name();

            if name.ends_with("/manifest.json") {
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() == 2 {
                    found_prefix = Some(format!("{}/", parts[0]));
                    break;
                }
            }
        }

        if let Some(prefix) = found_prefix {
            let manifest_path = format!("{}manifest.json", prefix);
            let mut file = archive
                .by_name(&manifest_path)
                .map_err(|e| format!("Failed to open {}: {}", manifest_path, e))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read {}: {}", manifest_path, e))?;
            return Ok((content, prefix));
        }

        Err("manifest.json not found in ZIP archive".to_string())
    }

    fn extract_zip_to_dir(
        &self,
        archive: &mut zip::ZipArchive<File>,
        prefix: &str,
        target_dir: &Path,
    ) -> Result<(), String> {
        const MAX_SINGLE_FILE_SIZE: u64 = 50 * 1024 * 1024;
        const MAX_TOTAL_SIZE: u64 = 200 * 1024 * 1024;
        let mut total_extracted_size: u64 = 0;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

            let name = file.name().to_string();

            if !prefix.is_empty() && !name.starts_with(prefix) {
                continue;
            }

            let relative_path = if prefix.is_empty() {
                name.clone()
            } else {
                name.strip_prefix(prefix).unwrap_or(&name).to_string()
            };

            if relative_path.is_empty() {
                continue;
            }

            let target_path = target_dir.join(&relative_path);

            let canonical_target = if target_path.exists() {
                target_path
                    .canonicalize()
                    .unwrap_or_else(|_| target_path.clone())
            } else if let Some(parent) = target_path.parent() {
                if parent.exists() {
                    let canonical_parent = parent
                        .canonicalize()
                        .unwrap_or_else(|_| parent.to_path_buf());
                    canonical_parent.join(target_path.file_name().unwrap_or_default())
                } else {
                    target_path.clone()
                }
            } else {
                target_path.clone()
            };
            let canonical_base = target_dir
                .canonicalize()
                .unwrap_or_else(|_| target_dir.to_path_buf());
            if !canonical_target.starts_with(&canonical_base) {
                return Err(format!(
                    "ZIP entry '{}' attempts path traversal outside target directory",
                    relative_path
                ));
            }

            let file_size = file.size();
            if file_size > MAX_SINGLE_FILE_SIZE {
                return Err(format!(
                    "ZIP entry '{}' exceeds max file size ({}MB > 50MB)",
                    file.name(),
                    file_size / 1024 / 1024
                ));
            }
            total_extracted_size += file_size;
            if total_extracted_size > MAX_TOTAL_SIZE {
                return Err(format!(
                    "ZIP total extracted size exceeds limit ({}MB > 200MB)",
                    total_extracted_size / 1024 / 1024
                ));
            }

            if file.is_dir() {
                fs::create_dir_all(&target_path).map_err(|e| {
                    format!("Failed to create directory {}: {}", target_path.display(), e)
                })?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                let mut outfile = File::create(&target_path).map_err(|e| {
                    format!("Failed to create file {}: {}", target_path.display(), e)
                })?;
                io::copy(&mut file, &mut outfile).map_err(|e| {
                    format!("Failed to write file {}: {}", target_path.display(), e)
                })?;
            }
        }

        Ok(())
    }

    pub fn get_plugin_settings(&self, plugin_id: &str) -> Result<serde_json::Value, String> {
        let plugin_info = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let plugin_path = PathBuf::from(&plugin_info.path);
        let settings_path = plugin_path.join("settings.json");

        if !settings_path.exists() {
            return Ok(serde_json::json!({}));
        }

        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;

        Ok(settings)
    }

    pub fn set_plugin_settings(
        &self,
        plugin_id: &str,
        settings: serde_json::Value,
    ) -> Result<(), String> {
        let plugin_info = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let plugin_path = PathBuf::from(&plugin_info.path);
        let settings_path = plugin_path.join("settings.json");

        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    pub fn get_plugin_icon(&self, plugin_id: &str) -> Result<String, String> {
        let plugin_info = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let plugin_path = PathBuf::from(&plugin_info.path);

        let icon_filename = plugin_info.manifest.icon.as_deref().unwrap_or("icon.png");

        if icon_filename.contains("..") || std::path::Path::new(icon_filename).is_absolute() {
            return Err(format!("Plugin icon path '{}' is not safe", icon_filename));
        }

        let icon_path = plugin_path.join(icon_filename);

        if !icon_path.exists() {
            return Ok(String::new());
        }

        let content =
            std::fs::read(&icon_path).map_err(|e| format!("Failed to read icon file: {}", e))?;

        let extension = icon_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mime_type = match extension {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "bmp" => "image/bmp",
            _ => "image/png",
        };

        if extension != "svg" && extension != "gif" {
            match image::load_from_memory(&content) {
                Ok(img) => {
                    let width = img.width();
                    let height = img.height();

                    if width != height {
                        return Err(format!("Icon must be square, got {}x{}", width, height));
                    }

                    if width > 2048 || height > 2048 {
                        return Err(format!(
                            "Icon size must not exceed 2048x2048, got {}x{}",
                            width, height
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to decode icon image: {}", e));
                }
            }
        }

        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&content);

        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    pub fn get_plugin_css(&self, plugin_id: &str) -> Result<String, String> {
        let plugin_info = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let plugin_path = PathBuf::from(&plugin_info.path);
        let css_path = plugin_path.join("style.css");

        if !css_path.exists() {
            return Ok(String::new());
        }

        std::fs::read_to_string(&css_path).map_err(|e| format!("Failed to read CSS file: {}", e))
    }

    pub fn get_all_plugin_css(&self) -> Result<Vec<(String, String)>, String> {
        let mut result = Vec::new();

        for (plugin_id, plugin_info) in &self.plugins {
            if matches!(plugin_info.state, PluginState::Enabled) {
                let plugin_path = PathBuf::from(&plugin_info.path);
                let css_path = plugin_path.join("style.css");
                if css_path.exists() {
                    if let Ok(css_content) = std::fs::read_to_string(&css_path) {
                        if !css_content.is_empty() {
                            result.push((plugin_id.clone(), css_content));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    pub fn delete_plugin(&mut self, plugin_id: &str, delete_data: bool) -> Result<(), String> {
        if let Some(plugin_info) = self.plugins.get(plugin_id) {
            if matches!(plugin_info.state, PluginState::Enabled) {
                return Err(format!(
                    "插件 '{}' 正在运行，请先禁用后再删除",
                    plugin_info.manifest.name
                ));
            }
        }

        let _dropped_runtime = {
            let mut runtimes = self.runtimes.write().unwrap_or_else(|e| {
                eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
                e.into_inner()
            });
            runtimes.remove(plugin_id)
        };

        drop(_dropped_runtime);

        let plugin_info = self
            .plugins
            .remove(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        let plugin_path = PathBuf::from(&plugin_info.path);
        if plugin_path.exists() {
            let mut last_error = None;
            for attempt in 0..3 {
                match fs::remove_dir_all(&plugin_path) {
                    Ok(_) => {
                        last_error = None;
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < 2 {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
            }
            if let Some(e) = last_error {
                return Err(format!("Failed to delete plugin directory: {}", e));
            }
        }

        let data_dir = self.data_dir.join(plugin_id);
        if data_dir.exists() {
            let should_delete = delete_data || {
                fs::read_dir(&data_dir)
                    .map(|mut e| e.next().is_none())
                    .unwrap_or(false)
            };
            if should_delete {
                let mut last_error = None;
                for attempt in 0..3 {
                    match fs::remove_dir_all(&data_dir) {
                        Ok(_) => {
                            last_error = None;
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                            if attempt < 2 {
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                        }
                    }
                }
                if let Some(e) = last_error {
                    return Err(format!("Failed to delete plugin data directory: {}", e));
                }
            }
        }

        Ok(())
    }

    pub fn is_newer_version(remote: &str, local: &str) -> bool {
        let parse_version =
            |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

        let remote_parts = parse_version(remote);
        let local_parts = parse_version(local);

        for i in 0..remote_parts.len().max(local_parts.len()) {
            let r = remote_parts.get(i).copied().unwrap_or(0);
            let l = local_parts.get(i).copied().unwrap_or(0);
            if r > l {
                return true;
            } else if r < l {
                return false;
            }
        }
        false
    }

    pub fn plugins(&self) -> &HashMap<String, PluginInfo> {
        &self.plugins
    }

    pub fn notify_page_changed(&self, path: &str) {
        let runtimes = self.runtimes.read().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        for (id, runtime) in runtimes.iter() {
            if let Some(info) = self.plugins.get(id) {
                if matches!(info.state, PluginState::Enabled) {
                    if let Err(e) = runtime.call_lifecycle_with_arg("onPageChanged", path) {
                        eprintln!("[WARN] Failed to call onPageChanged for plugin '{}': {}", id, e);
                    }
                }
            }
        }
    }

    pub fn notify_locale_changed(&self, locale: &str) {
        let runtimes = self.runtimes.read().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        for (id, runtime) in runtimes.iter() {
            if let Some(info) = self.plugins.get(id) {
                if matches!(info.state, PluginState::Enabled) {
                    if let Err(e) = runtime.call_lifecycle_with_arg("onLocaleChanged", locale) {
                        eprintln!(
                            "[WARN] Failed to call onLocaleChanged for plugin '{}': {}",
                            id, e
                        );
                        let _ = emit_log_event(
                            id,
                            "error",
                            &format!("Failed to call onLocaleChanged: {}", e),
                        );
                    }
                }
            }
        }
    }
}
