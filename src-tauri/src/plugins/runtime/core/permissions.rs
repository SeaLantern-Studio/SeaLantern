use super::runtime::PluginRuntime;
use mlua::Table;

impl PluginRuntime {
    #[allow(dead_code)] // fu*k fmt
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
        let module_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create {} table: {}", module_name, e))?;

        let module_name_owned = module_name.to_string();
        let error_fn = self
            .lua
            .create_function(move |_, ()| -> Result<(), mlua::Error> {
                Err(mlua::Error::runtime(format!(
                    "权限不足: 使用 'sl.{}' 模块需要在 manifest.json 中声明 '{}' 权限",
                    module_name_owned, module_name_owned
                )))
            })
            .map_err(|e| format!("Failed to create error function for {}: {}", module_name, e))?;

        module_table
            .set("_error", error_fn)
            .map_err(|e| format!("Failed to set error for {}: {}", module_name, e))?;

        let module_name_for_meta = module_name.to_string();
        let meta_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create metatable for {}: {}", module_name, e))?;

        let index_fn = self
            .lua
            .create_function(move |_, _key: mlua::Value| -> Result<mlua::Value, mlua::Error> {
                Err(mlua::Error::runtime(format!(
                    "权限不足: 使用 'sl.{}' 模块需要在 manifest.json 中声明 '{}' 权限",
                    module_name_for_meta, module_name_for_meta
                )))
            })
            .map_err(|e| format!("Failed to create __index for {}: {}", module_name, e))?;

        meta_table
            .set(mlua::MetaMethod::Index.name(), index_fn)
            .map_err(|e| format!("Failed to set __index for {}: {}", module_name, e))?;

        module_table.set_metatable(Some(meta_table));

        sl.set(module_name, module_table)
            .map_err(|e| format!("Failed to set sl.{}: {}", module_name, e))?;

        Ok(())
    }
}
