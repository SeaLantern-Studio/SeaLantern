use super::common::{
    check_server_permission, checked_file_metadata, create_server_entry, map_lua_err, with_server,
    with_server_path, ServerContext,
};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn list(lua: &Lua, ctx: &ServerContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, ()| {
        check_server_permission(&ctx.permissions)?;
        let servers = crate::services::global::server_manager().get_server_list();
        let result = lua.create_table()?;
        for (i, server) in servers.iter().enumerate() {
            result.set(i + 1, create_server_entry(lua, server)?)?;
        }
        Ok(result)
    })
    .map_err(|e| map_lua_err("server.create_list_failed", e))
}

pub(super) fn get_path(lua: &Lua, ctx: &ServerContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, server_id: String| {
        check_server_permission(&ctx.permissions)?;
        with_server(&server_id, |server| Ok(server.path.clone()))
    })
    .map_err(|e| map_lua_err("server.create_get_path_failed", e))
}

pub(super) fn read_file(lua: &Lua, ctx: &ServerContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (server_id, relative_path): (String, String)| {
        check_server_permission(&ctx.permissions)?;
        with_server_path(&server_id, &relative_path, |_, full_path| {
            checked_file_metadata(&full_path)?;

            fs::read_to_string(&full_path).map_err(|e| {
                mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                    "server.failed_to_read_file",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                ))
            })
        })
    })
    .map_err(|e| map_lua_err("server.create_read_file_failed", e))
}

pub(super) fn write_file(lua: &Lua, ctx: &ServerContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (server_id, relative_path, content): (String, String, String)| {
        check_server_permission(&ctx.permissions)?;
        with_server_path(&server_id, &relative_path, |_, full_path| {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                        "server.failed_to_create_dir",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })?;
            }

            fs::write(&full_path, &content).map_err(|e| {
                mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                    "server.failed_to_write_file",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                ))
            })?;
            Ok(true)
        })
    })
    .map_err(|e| map_lua_err("server.create_write_file_failed", e))
}

pub(super) fn list_dir(lua: &Lua, ctx: &ServerContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, (server_id, relative_path): (String, String)| {
        check_server_permission(&ctx.permissions)?;
        with_server_path(&server_id, &relative_path, |_, full_path| {
            if !full_path.is_dir() {
                return Err(mlua::Error::runtime(
                    crate::services::global::i18n_service().t("server.path_not_directory"),
                ));
            }

            let entries = fs::read_dir(&full_path).map_err(|e| {
                mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                    "server.failed_to_read_dir",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                ))
            })?;

            let result = lua.create_table()?;
            for (i, entry) in entries.enumerate() {
                let entry = entry.map_err(|e| {
                    mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                        "server.failed_to_read_entry",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })?;
                let metadata = entry.metadata().map_err(|e| {
                    mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                        "server.failed_to_get_metadata",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })?;

                let item = lua.create_table()?;
                item.set("name", entry.file_name().to_string_lossy().to_string())?;
                item.set("is_dir", metadata.is_dir())?;
                item.set("size", metadata.len())?;
                result.set(i + 1, item)?;
            }
            Ok(result)
        })
    })
    .map_err(|e| map_lua_err("server.create_list_dir_failed", e))
}

pub(super) fn exists(lua: &Lua, ctx: &ServerContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (server_id, relative_path): (String, String)| {
        check_server_permission(&ctx.permissions)?;
        with_server_path(&server_id, &relative_path, |_, full_path| Ok(full_path.exists()))
    })
    .map_err(|e| map_lua_err("server.create_exists_failed", e))
}
