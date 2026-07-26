//! Lua 插件执行引擎。
//!
//! 这里仅提供受限 Lua、存储和日志能力。其余宿主能力必须通过未来的显式 trait 接入。

mod storage;

use std::{
    fs,
    path::{Path, PathBuf},
};

use mlua::{Lua, LuaOptions, StdLib, Table, Value};

use crate::app_plugin::{AppPluginError, PluginManifest, PluginPermission};
use crate::observability;

use self::storage::PluginStorage;

pub struct PluginEngine {
    lua: Lua,
    plugin_id: String,
    plugin_dir: PathBuf,
    main: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Lifecycle {
    Load,
    Enable,
    Disable,
    Unload,
}

impl Lifecycle {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Load => "on_load",
            Self::Enable => "on_enable",
            Self::Disable => "on_disable",
            Self::Unload => "on_unload",
        }
    }
}

impl PluginEngine {
    pub(crate) fn new(
        manifest: &PluginManifest,
        plugin_dir: &Path,
        data_dir: &Path,
    ) -> Result<Self, AppPluginError> {
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8 | StdLib::COROUTINE,
            LuaOptions::default(),
        )
        .map_err(engine_error)?;
        let engine = Self {
            lua,
            plugin_id: manifest.id.clone(),
            plugin_dir: plugin_dir.to_path_buf(),
            main: manifest.main.clone(),
        };
        engine.install_sl(manifest, data_dir)?;
        Ok(engine)
    }

    pub(crate) fn load(&self) -> Result<(), AppPluginError> {
        let path = resolve_main_path(&self.plugin_dir, &self.main)?;
        let source = fs::read(&path).map_err(|error| {
            AppPluginError::Engine(format!(
                "failed to read plugin entry script {}: {error}",
                path.display()
            ))
        })?;
        let source =
            String::from_utf8_lossy(source.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&source));
        self.lua
            .load(source.as_ref())
            .set_name(path.to_string_lossy().as_ref())
            .exec()
            .map_err(|error| {
                AppPluginError::Engine(format!(
                    "failed to execute plugin entry script {}: {error}",
                    path.display()
                ))
            })
    }

    pub(crate) fn call_lifecycle(&self, lifecycle: Lifecycle) -> Result<(), AppPluginError> {
        let name = lifecycle.as_str();
        match self
            .lua
            .globals()
            .get::<Value>(name)
            .map_err(engine_error)?
        {
            Value::Nil => Ok(()),
            Value::Function(function) => function.call::<()>(()).map_err(|error| {
                AppPluginError::Engine(format!(
                    "plugin lifecycle callback '{name}' failed: {error}"
                ))
            }),
            value => Err(AppPluginError::Engine(format!(
                "plugin lifecycle callback '{name}' must be a function, got {}",
                value.type_name()
            ))),
        }
    }

    fn install_sl(&self, manifest: &PluginManifest, data_dir: &Path) -> Result<(), AppPluginError> {
        let sl = self.lua.create_table().map_err(engine_error)?;
        self.install_storage(&sl, manifest, data_dir)?;
        self.install_log(&sl, manifest)?;
        self.lua.globals().set("sl", sl).map_err(engine_error)
    }

    fn install_storage(
        &self,
        sl: &Table,
        manifest: &PluginManifest,
        data_dir: &Path,
    ) -> Result<(), AppPluginError> {
        let table = self.lua.create_table().map_err(engine_error)?;
        let permitted = manifest.permissions.contains(&PluginPermission::Storage);
        let storage = PluginStorage::new(&self.plugin_id, data_dir);

        let context = storage.clone();
        table
            .set(
                "get",
                self.lua
                    .create_function(move |lua, key: String| {
                        require_permission(permitted, "storage")?;
                        context.get(lua, key)
                    })
                    .map_err(engine_error)?,
            )
            .map_err(engine_error)?;

        let context = storage.clone();
        table
            .set(
                "keys",
                self.lua
                    .create_function(move |lua, ()| {
                        require_permission(permitted, "storage")?;
                        context.keys(lua)
                    })
                    .map_err(engine_error)?,
            )
            .map_err(engine_error)?;

        let context = storage.clone();
        table
            .set(
                "set",
                self.lua
                    .create_function(move |_, (key, value): (String, Value)| {
                        require_permission(permitted, "storage")?;
                        context.set(key, value)
                    })
                    .map_err(engine_error)?,
            )
            .map_err(engine_error)?;

        table
            .set(
                "remove",
                self.lua
                    .create_function(move |_, key: String| {
                        require_permission(permitted, "storage")?;
                        storage.remove(key)
                    })
                    .map_err(engine_error)?,
            )
            .map_err(engine_error)?;
        sl.set("storage", table).map_err(engine_error)
    }

    fn install_log(&self, sl: &Table, manifest: &PluginManifest) -> Result<(), AppPluginError> {
        let table = self.lua.create_table().map_err(engine_error)?;
        let permitted = manifest.permissions.contains(&PluginPermission::Log);
        for (name, level) in
            [("debug", "debug"), ("info", "info"), ("warn", "warn"), ("error", "error")]
        {
            let plugin_id = self.plugin_id.clone();
            table
                .set(
                    name,
                    self.lua
                        .create_function(move |_, message: String| {
                            require_permission(permitted, "log")?;
                            emit_log(level, &plugin_id, &message);
                            Ok(())
                        })
                        .map_err(engine_error)?,
                )
                .map_err(engine_error)?;
        }
        sl.set("log", table).map_err(engine_error)
    }
}

fn resolve_main_path(plugin_dir: &Path, main: &str) -> Result<PathBuf, AppPluginError> {
    let relative = Path::new(main);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(AppPluginError::Engine(
            "plugin entry script must remain inside the plugin directory".to_owned(),
        ));
    }
    let base = plugin_dir.canonicalize().map_err(|error| {
        AppPluginError::Engine(format!("failed to resolve plugin directory: {error}"))
    })?;
    let script = base.join(relative).canonicalize().map_err(|error| {
        AppPluginError::Engine(format!("failed to resolve plugin entry script {main}: {error}"))
    })?;
    if script.starts_with(&base) {
        Ok(script)
    } else {
        Err(AppPluginError::Engine(
            "plugin entry script must remain inside the plugin directory".to_owned(),
        ))
    }
}

fn require_permission(permitted: bool, permission: &'static str) -> mlua::Result<()> {
    permitted.then_some(()).ok_or_else(|| {
        mlua::Error::runtime(format!("plugin does not have the required '{permission}' permission"))
    })
}

fn engine_error(error: impl std::fmt::Display) -> AppPluginError {
    AppPluginError::Engine(error.to_string())
}

fn emit_log(level: &str, plugin_id: &str, message: &str) {
    match level {
        "debug" => {
            tracing::debug!(target: observability::APP_PLUGIN_TARGET, event_name = observability::EVENT_APP_PLUGIN_LOG_EMITTED, plugin_id, level, message, "plugin emitted a log message")
        }
        "info" => {
            tracing::info!(target: observability::APP_PLUGIN_TARGET, event_name = observability::EVENT_APP_PLUGIN_LOG_EMITTED, plugin_id, level, message, "plugin emitted a log message")
        }
        "warn" => {
            tracing::warn!(target: observability::APP_PLUGIN_TARGET, event_name = observability::EVENT_APP_PLUGIN_LOG_EMITTED, plugin_id, level, message, "plugin emitted a log message")
        }
        _ => {
            tracing::error!(target: observability::APP_PLUGIN_TARGET, event_name = observability::EVENT_APP_PLUGIN_LOG_EMITTED, plugin_id, level, message, "plugin emitted a log message")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn test_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("sealantern-extra-engine-{label}-{}-{nonce}", std::process::id()))
    }

    fn manifest(permissions: Vec<PluginPermission>) -> PluginManifest {
        PluginManifest {
            api_version: 2,
            id: "example.plugin".to_string(),
            name: "Example".to_string(),
            version: "1.0.0".to_string(),
            main: "main.lua".to_string(),
            permissions,
        }
    }

    #[test]
    fn sandbox_excludes_os_and_io() {
        let root = test_dir("sandbox");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new(&manifest(vec![]), &root, &root.join("data"))
            .expect("engine should initialize");

        let os: Value = engine
            .lua
            .load("return os")
            .eval()
            .expect("Lua should evaluate");
        let io: Value = engine
            .lua
            .load("return io")
            .eval()
            .expect("Lua should evaluate");
        assert!(matches!(os, Value::Nil));
        assert!(matches!(io, Value::Nil));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn storage_round_trips_lua_arrays() {
        let root = test_dir("storage");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new(
            &manifest(vec![PluginPermission::Storage]),
            &root,
            &root.join("data"),
        )
        .expect("engine should initialize");

        engine
            .lua
            .load(
                r#"
                    sl.storage.set("items", { "first", "second" })
                    local items = sl.storage.get("items")
                    assert(items[1] == "first")
                    assert(items[2] == "second")
                "#,
            )
            .exec()
            .expect("storage array should round-trip");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn storage_requires_declared_permission() {
        let root = test_dir("permission");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new(&manifest(vec![]), &root, &root.join("data"))
            .expect("engine should initialize");

        let allowed: bool = engine
            .lua
            .load("return pcall(function() sl.storage.keys() end)")
            .eval()
            .expect("Lua should evaluate");
        assert!(!allowed);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn lifecycle_uses_top_level_snake_case_callbacks() {
        let root = test_dir("lifecycle");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        fs::write(root.join("main.lua"), "function on_load() lifecycle_loaded = true end")
            .expect("entry script should be written");
        let engine = PluginEngine::new(&manifest(vec![]), &root, &root.join("data"))
            .expect("engine should initialize");

        engine.load().expect("entry script should load");
        engine
            .call_lifecycle(Lifecycle::Load)
            .expect("on_load should run");
        let loaded: bool = engine
            .lua
            .load("return lifecycle_loaded")
            .eval()
            .expect("Lua should evaluate");
        assert!(loaded);

        let _ = fs::remove_dir_all(root);
    }
}
