use crate::services::global::i18n_service;
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub(super) const ELEMENT_GET_TIMEOUT_MS: u64 = 500;

pub(super) fn convert_lua_string(s: &mlua::String) -> String {
    String::from_utf8_lossy(&s.as_bytes()).into_owned()
}

pub(super) fn wait_for_element_response(
    lua: &mlua::Lua,
    rx: Receiver<String>,
) -> Result<mlua::Value, mlua::Error> {
    match rx.recv_timeout(Duration::from_millis(ELEMENT_GET_TIMEOUT_MS)) {
        Ok(val) => Ok(mlua::Value::String(lua.create_string(&val).map_err(mlua::Error::external)?)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(mlua::Value::Nil),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(mlua::Value::Nil),
    }
}

pub(super) fn element_create_error(key: &str, err: &mlua::Error) -> String {
    i18n_service()
        .t_with_options(key, &crate::plugins::runtime::console::i18n_arg("0", &err.to_string()))
}

pub(super) fn element_set_error(key: &str, err: &mlua::Error) -> String {
    i18n_service()
        .t_with_options(key, &crate::plugins::runtime::console::i18n_arg("0", &err.to_string()))
}

pub(super) fn log_element_action_error(key: &str, err: &dyn std::fmt::Display) {
    eprintln!(
        "[Element] {}",
        i18n_service().t_with_options(
            key,
            &crate::plugins::runtime::console::i18n_arg("0", &err.to_string()),
        )
    );
}
