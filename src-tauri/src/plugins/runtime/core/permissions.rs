use super::runtime::PluginRuntime;
use crate::hardcode_data::plugin_manifest::missing_permission_in_manifest_message;
use mlua::Table;

impl PluginRuntime {
    #[allow(dead_code)] // 未来校验调用
    pub(super) fn check_permission(&self, permission: &str) -> mlua::Result<()> {
        if self.permissions.iter().any(|p| p == permission) {
            Ok(())
        } else {
            Err(mlua::Error::runtime(format!(
                "Permission denied: '{}' permission is required for this operation",
                permission
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
                format!(
                    "权限不足: 使用 'sl.{}' 模块需要在 manifest.json 中声明 '{}' 权限",
                    namespace_name, required_permission
                )
            }
        };

        let module_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create {} table: {}", namespace_name, e))?;

        let missing_message = missing_permission_message();
        let error_fn = self
            .lua
            .create_function(move |_, ()| -> Result<(), mlua::Error> {
                Err(mlua::Error::runtime(missing_message.clone()))
            })
            .map_err(|e| {
                format!("Failed to create error function for {}: {}", namespace_name, e)
            })?;

        module_table
            .set("_error", error_fn)
            .map_err(|e| format!("Failed to set error for {}: {}", namespace_name, e))?;

        let missing_message_for_meta = missing_permission_message();
        let meta_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create metatable for {}: {}", namespace_name, e))?;

        let index_fn = self
            .lua
            .create_function(move |_, _key: mlua::Value| -> Result<mlua::Value, mlua::Error> {
                Err(mlua::Error::runtime(missing_message_for_meta.clone()))
            })
            .map_err(|e| format!("Failed to create __index for {}: {}", namespace_name, e))?;

        meta_table
            .set(mlua::MetaMethod::Index.name(), index_fn)
            .map_err(|e| format!("Failed to set __index for {}: {}", namespace_name, e))?;

        module_table.set_metatable(Some(meta_table));

        sl.set(namespace_name, module_table)
            .map_err(|e| format!("Failed to set sl.{}: {}", namespace_name, e))?;

        Ok(())
    }
}
