//! Lua 插件执行引擎。
//!
//! 这里仅提供受限 Lua、存储和日志能力。其余宿主能力必须通过未来的显式 trait 接入。

mod storage;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use mlua::{HookTriggers, Lua, LuaOptions, StdLib, Table, Value, VmState};
use sealantern_core::app_plugin::{
    CapabilityDispatchError, CapabilityDispatcher, CapabilityId, CapabilityInvocation,
    ExecutionPrincipal, ScopeBinding, ScopeKind, TrustSource,
};
use serde_json::Value as JsonValue;

use crate::app_plugin::{AppPluginError, PluginManifest};
use crate::observability;

use self::storage::{json_to_lua, lua_to_json, PluginStorage};

const EXECUTION_HOOK_INTERVAL: u32 = 1_000;
const MAX_EXECUTION_INSTRUCTIONS: u64 = 1_000_000;
const MAX_LUA_MEMORY_BYTES: usize = 8 * 1024 * 1024;
const EXECUTION_LIMIT_MESSAGE: &str = "plugin execution instruction budget exhausted";

pub struct PluginEngine {
    lua: Lua,
    plugin_id: String,
    plugin_dir: PathBuf,
    main: String,
    execution_budget: Arc<Mutex<Option<u64>>>,
    dispatcher: Option<Arc<dyn CapabilityDispatcher>>,
    runtime_handle: Option<tokio::runtime::Handle>,
    trust_source: TrustSource,
    direct_storage_enabled: bool,
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
    #[cfg(test)]
    pub(crate) fn new(
        manifest: &PluginManifest,
        plugin_dir: &Path,
        data_dir: &Path,
    ) -> Result<Self, AppPluginError> {
        Self::new_inner(manifest, plugin_dir, data_dir, None, TrustSource::UntrustedLocal, true)
    }

    pub(crate) fn new_with_dispatcher(
        manifest: &PluginManifest,
        plugin_dir: &Path,
        data_dir: &Path,
        dispatcher: Option<Arc<dyn CapabilityDispatcher>>,
        trust_source: TrustSource,
    ) -> Result<Self, AppPluginError> {
        Self::new_inner(manifest, plugin_dir, data_dir, dispatcher, trust_source, false)
    }

    fn new_inner(
        manifest: &PluginManifest,
        plugin_dir: &Path,
        data_dir: &Path,
        dispatcher: Option<Arc<dyn CapabilityDispatcher>>,
        trust_source: TrustSource,
        direct_storage_enabled: bool,
    ) -> Result<Self, AppPluginError> {
        // Hook 只作用于当前 Lua 线程，因此首版不暴露 coroutine 以避免绕过执行预算。
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )
        .map_err(engine_error)?;
        lua.set_memory_limit(MAX_LUA_MEMORY_BYTES)
            .map_err(engine_error)?;
        let execution_budget = Arc::new(Mutex::new(None));
        let hook_budget = Arc::clone(&execution_budget);
        lua.set_hook(
            HookTriggers::new().every_nth_instruction(EXECUTION_HOOK_INTERVAL),
            move |_, _| {
                let mut remaining = hook_budget.lock().map_err(|_| {
                    mlua::Error::runtime("plugin execution budget lock is poisoned")
                })?;
                let Some(remaining) = remaining.as_mut() else {
                    return Ok(VmState::Continue);
                };
                if *remaining <= u64::from(EXECUTION_HOOK_INTERVAL) {
                    return Err(mlua::Error::runtime(EXECUTION_LIMIT_MESSAGE));
                }
                *remaining -= u64::from(EXECUTION_HOOK_INTERVAL);
                Ok(VmState::Continue)
            },
        );
        let engine = Self {
            lua,
            plugin_id: manifest.id.clone(),
            plugin_dir: plugin_dir.to_path_buf(),
            main: manifest.main.clone(),
            execution_budget,
            dispatcher,
            runtime_handle: tokio::runtime::Handle::try_current().ok(),
            trust_source,
            direct_storage_enabled,
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
        self.run_with_execution_budget("entry_script", || {
            self.lua
                .load(source.as_ref())
                .set_name(path.to_string_lossy().as_ref())
                .exec()
        })
    }

    pub(crate) fn call_lifecycle(&self, lifecycle: Lifecycle) -> Result<(), AppPluginError> {
        let name = lifecycle.as_str();
        self.run_with_execution_budget(name, || match self.lua.globals().get::<Value>(name)? {
            Value::Nil => Ok(()),
            Value::Function(function) => function.call::<()>(()),
            value => Err(mlua::Error::runtime(format!(
                "plugin lifecycle callback '{name}' must be a function, got {}",
                value.type_name()
            ))),
        })
    }

    fn install_sl(&self, manifest: &PluginManifest, data_dir: &Path) -> Result<(), AppPluginError> {
        let sl = self.lua.create_table().map_err(engine_error)?;
        self.install_storage(&sl, manifest, data_dir)?;
        self.install_log(&sl, manifest)?;
        self.install_capabilities(&sl, manifest)?;
        self.lua.globals().set("sl", sl).map_err(engine_error)
    }

    fn install_storage(
        &self,
        sl: &Table,
        manifest: &PluginManifest,
        data_dir: &Path,
    ) -> Result<(), AppPluginError> {
        let table = self.lua.create_table().map_err(engine_error)?;
        let read_permitted = self.direct_storage_enabled
            && manifest
                .capabilities
                .iter()
                .any(|capability| capability.id == "plugin.storage.read");
        let write_permitted = self.direct_storage_enabled
            && manifest
                .capabilities
                .iter()
                .any(|capability| capability.id == "plugin.storage.write");
        let storage = PluginStorage::new(&self.plugin_id, data_dir);

        let context = storage.clone();
        table
            .set(
                "get",
                self.lua
                    .create_function(move |lua, key: String| {
                        require_permission(read_permitted, "storage read")?;
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
                        require_permission(read_permitted, "storage read")?;
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
                        require_permission(write_permitted, "storage write")?;
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
                        require_permission(write_permitted, "storage write")?;
                        storage.remove(key)
                    })
                    .map_err(engine_error)?,
            )
            .map_err(engine_error)?;
        sl.set("storage", table).map_err(engine_error)
    }

    fn install_log(&self, sl: &Table, manifest: &PluginManifest) -> Result<(), AppPluginError> {
        let table = self.lua.create_table().map_err(engine_error)?;
        let permitted = manifest
            .capabilities
            .iter()
            .any(|capability| capability.id == "plugin.log.emit");
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

    fn install_capabilities(
        &self,
        sl: &Table,
        manifest: &PluginManifest,
    ) -> Result<(), AppPluginError> {
        let table = self.lua.create_table().map_err(engine_error)?;
        let Some(dispatcher) = self.dispatcher.as_ref().cloned() else {
            sl.set("capabilities", table).map_err(engine_error)?;
            return Ok(());
        };
        let Some(handle) = self.runtime_handle.as_ref().cloned() else {
            return Err(AppPluginError::Engine(
                "plugin capability dispatcher requires a Tokio runtime".to_string(),
            ));
        };
        let plugin_id = self.plugin_id.clone();
        let trust_source = self.trust_source;
        let declarations = manifest.capabilities.clone();
        table
            .set(
                "invoke",
                self.lua
                    .create_function(
                        move |lua,
                              (id, payload, scope, session_id, approval_token): (
                            String,
                            Option<Value>,
                            Option<Value>,
                            Option<String>,
                            Option<String>,
                        )| {
                            let capability = CapabilityId::new(&id)
                                .map_err(|error| mlua::Error::runtime(error.to_string()))?;
                            let scope = parse_scope(scope)?;
                            let declared = declarations.iter().any(|declaration| {
                                declaration.id == id && declaration.scope.as_ref() == scope.as_ref()
                            });
                            let payload = payload
                                .map(|value| lua_to_json(&value, 0))
                                .transpose()?
                                .unwrap_or(JsonValue::Null);
                            let invocation = CapabilityInvocation {
                                principal: ExecutionPrincipal::Plugin(plugin_id.clone()),
                                trust_source,
                                capability,
                                scope,
                                declared,
                                session_id,
                                payload,
                                approval_token,
                                request_id: uuid::Uuid::new_v4().to_string(),
                            };
                            let response = handle
                                .block_on(dispatcher.invoke(invocation))
                                .map_err(dispatcher_error)?;
                            json_to_lua(lua, &response, 0)
                        },
                    )
                    .map_err(engine_error)?,
            )
            .map_err(engine_error)?;
        sl.set("capabilities", table).map_err(engine_error)
    }

    fn run_with_execution_budget<T>(
        &self,
        operation: &'static str,
        action: impl FnOnce() -> mlua::Result<T>,
    ) -> Result<T, AppPluginError> {
        {
            let mut budget = self.execution_budget.lock().map_err(|_| {
                AppPluginError::Engine("plugin execution budget lock is poisoned".to_string())
            })?;
            *budget = Some(MAX_EXECUTION_INSTRUCTIONS);
        }

        let result = action();
        {
            let mut budget = self.execution_budget.lock().map_err(|_| {
                AppPluginError::Engine("plugin execution budget lock is poisoned".to_string())
            })?;
            *budget = None;
        }

        result.map_err(|error| {
            if error.to_string().contains(EXECUTION_LIMIT_MESSAGE) {
                observability::app_plugin_execution_limit_exceeded(&self.plugin_id, operation);
                AppPluginError::ExecutionLimit { operation }
            } else {
                engine_error(error)
            }
        })
    }
}

fn parse_scope(value: Option<Value>) -> mlua::Result<Option<ScopeBinding>> {
    let Some(Value::Table(table)) = value else {
        return Ok(None);
    };
    let kind: String = table.get("kind")?;
    let value: String = table.get("value")?;
    let kind = match kind.as_str() {
        "plugin_data" => ScopeKind::PluginData,
        "plugin_bundle" => ScopeKind::PluginBundle,
        "server_instance" => ScopeKind::ServerInstance,
        "app_global" => ScopeKind::AppGlobal,
        "network_origin" => ScopeKind::NetworkOrigin,
        "ui_extension" => ScopeKind::UiExtension,
        "host_element" => ScopeKind::HostElement,
        "market_artifact" => ScopeKind::MarketArtifact,
        "approved_executable" => ScopeKind::ApprovedExecutable,
        _ => return Err(mlua::Error::runtime("plugin capability scope kind is invalid")),
    };
    ScopeBinding::new(kind, value)
        .map(Some)
        .map_err(|error| mlua::Error::runtime(error.to_string()))
}

fn dispatcher_error(error: CapabilityDispatchError) -> mlua::Error {
    mlua::Error::runtime(error.to_string())
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

    fn manifest(capabilities: Vec<crate::app_plugin::PluginCapability>) -> PluginManifest {
        PluginManifest {
            api_version: 2,
            id: "example.plugin".to_string(),
            name: "Example".to_string(),
            version: "1.0.0".to_string(),
            main: "main.lua".to_string(),
            capabilities,
        }
    }

    #[test]
    fn sandbox_excludes_os_io_and_coroutines() {
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
        let coroutine: Value = engine
            .lua
            .load("return coroutine")
            .eval()
            .expect("Lua should evaluate");
        assert!(matches!(os, Value::Nil));
        assert!(matches!(io, Value::Nil));
        assert!(matches!(coroutine, Value::Nil));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn storage_round_trips_lua_arrays() {
        let root = test_dir("storage");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new(
            &manifest(vec![
                crate::app_plugin::PluginCapability::new("plugin.storage.read"),
                crate::app_plugin::PluginCapability::new("plugin.storage.write"),
            ]),
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

    #[tokio::test]
    async fn production_engine_does_not_expose_direct_storage() {
        struct Dispatcher;

        #[async_trait::async_trait]
        impl CapabilityDispatcher for Dispatcher {
            async fn invoke(
                &self,
                _: CapabilityInvocation,
            ) -> Result<JsonValue, CapabilityDispatchError> {
                Err(CapabilityDispatchError::Unavailable("not used"))
            }
        }

        let root = test_dir("managed-storage");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new_with_dispatcher(
            &manifest(vec![crate::app_plugin::PluginCapability::new("plugin.storage.read")]),
            &root,
            &root.join("data"),
            Some(Arc::new(Dispatcher)),
            TrustSource::UntrustedLocal,
        )
        .expect("engine should initialize");

        let allowed: bool = engine
            .lua
            .load("return pcall(function() sl.storage.get('key') end)")
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

    #[test]
    fn instruction_budget_interrupts_non_terminating_entry_scripts() {
        let root = test_dir("instruction-budget");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        fs::write(root.join("main.lua"), "while true do end")
            .expect("entry script should be written");
        let engine = PluginEngine::new(&manifest(vec![]), &root, &root.join("data"))
            .expect("engine should initialize");

        assert!(matches!(
            engine.load(),
            Err(AppPluginError::ExecutionLimit { operation: "entry_script" })
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn memory_limit_rejects_oversized_lua_allocations() {
        let root = test_dir("memory-limit");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new(&manifest(vec![]), &root, &root.join("data"))
            .expect("engine should initialize");
        let script = format!(
            "return pcall(function() return string.rep('x', {}) end)",
            MAX_LUA_MEMORY_BYTES * 2
        );

        let allocated: bool = engine
            .lua
            .load(&script)
            .eval()
            .expect("Lua should evaluate");
        assert!(!allocated);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn storage_rejects_values_larger_than_its_quota() {
        let root = test_dir("storage-quota");
        fs::create_dir_all(&root).expect("plugin directory should be created");
        let engine = PluginEngine::new(
            &manifest(vec![crate::app_plugin::PluginCapability::new("plugin.storage.write")]),
            &root,
            &root.join("data"),
        )
        .expect("engine should initialize");

        let allowed: bool = engine
            .lua
            .load("return pcall(function() sl.storage.set('large', string.rep('x', 262145)) end)")
            .eval()
            .expect("Lua should evaluate");
        assert!(!allowed);

        let _ = fs::remove_dir_all(root);
    }
}
