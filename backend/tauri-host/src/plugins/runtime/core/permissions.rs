use super::runtime::PluginRuntime;
use crate::hardcode_data::plugin_manifest::missing_permission_in_manifest_message;
use crate::services::global::i18n_service;
use mlua::Table;
use std::collections::HashMap;

fn perm_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

fn perm_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

impl PluginRuntime {
    #[allow(dead_code)] // 未来校验调用
    pub(super) fn check_permission(&self, permission: &str) -> mlua::Result<()> {
        if self.permissions.iter().any(|p| p == permission) {
            Ok(())
        } else {
            Err(mlua::Error::runtime(perm_t1(
                "plugins.runtime.permissions.permission_required",
                permission,
            )))
        }
    }

    pub(super) fn setup_permission_denied_module(
        &self,
        sl: &Table,
        module_name: &str,
    ) -> Result<(), String> {
        self.setup_permission_denied_namespace(sl, module_name, module_name)
    }

    pub(super) fn setup_permission_denied_namespace(
        &self,
        sl: &Table,
        namespace_name: &str,
        required_permission: &str,
    ) -> Result<(), String> {
        let missing_permission_message = || {
            if namespace_name == required_permission {
                missing_permission_in_manifest_message(namespace_name)
            } else {
                perm_t2(
                    "plugins.runtime.permissions.namespace_permission_missing",
                    namespace_name,
                    required_permission,
                )
            }
        };

        let module_table = self.lua.create_table().map_err(|e| {
            perm_t2(
                "plugins.runtime.permissions.create_table_failed",
                namespace_name,
                e.to_string(),
            )
        })?;

        let missing_message = missing_permission_message();
        let error_fn = self
            .lua
            .create_function(move |_, ()| -> Result<(), mlua::Error> {
                Err(mlua::Error::runtime(missing_message.clone()))
            })
            .map_err(|e| {
                perm_t2(
                    "plugins.runtime.permissions.create_error_function_failed",
                    namespace_name,
                    e.to_string(),
                )
            })?;

        module_table.set("_error", error_fn).map_err(|e| {
            perm_t2("plugins.runtime.permissions.set_error_failed", namespace_name, e.to_string())
        })?;

        let missing_message_for_meta = missing_permission_message();
        let meta_table = self.lua.create_table().map_err(|e| {
            perm_t2(
                "plugins.runtime.permissions.create_metatable_failed",
                namespace_name,
                e.to_string(),
            )
        })?;

        let index_fn = self
            .lua
            .create_function(move |_, _key: mlua::Value| -> Result<mlua::Value, mlua::Error> {
                Err(mlua::Error::runtime(missing_message_for_meta.clone()))
            })
            .map_err(|e| {
                perm_t2(
                    "plugins.runtime.permissions.create_index_failed",
                    namespace_name,
                    e.to_string(),
                )
            })?;

        meta_table
            .set(mlua::MetaMethod::Index.name(), index_fn)
            .map_err(|e| {
                perm_t2(
                    "plugins.runtime.permissions.set_index_failed",
                    namespace_name,
                    e.to_string(),
                )
            })?;

        module_table.set_metatable(Some(meta_table));

        sl.set(namespace_name, module_table).map_err(|e| {
            perm_t2(
                "plugins.runtime.permissions.set_namespace_failed",
                namespace_name,
                e.to_string(),
            )
        })?;

        Ok(())
    }
}
