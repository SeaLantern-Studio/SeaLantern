use super::PluginRuntime;
use crate::services::global::i18n_service;
use mlua::Table;
use std::sync::mpsc::Receiver;
use std::time::Duration;

const ELEMENT_GET_TIMEOUT_MS: u64 = 500;

fn wait_for_element_response(
    lua: &mlua::Lua,
    rx: Receiver<String>,
) -> Result<mlua::Value, mlua::Error> {
    match rx.recv_timeout(Duration::from_millis(ELEMENT_GET_TIMEOUT_MS)) {
        Ok(val) => Ok(mlua::Value::String(lua.create_string(&val).map_err(mlua::Error::external)?)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(mlua::Value::Nil),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(mlua::Value::Nil),
    }
}

impl PluginRuntime {
    pub(super) fn setup_element_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::{element_response_create, emit_permission_log, emit_ui_event};

        let element_table = self.lua.create_table().map_err(|e| {
            i18n_service().t_with_options(
                "element.create_table_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let plugin_id = self.plugin_id.clone();

        let pid = plugin_id.clone();
        let get_text_fn = self
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.get_text", &selector);

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_text", &selector, &data) {
                    Ok(()) => wait_for_element_response(lua, rx),
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_get_text_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("get_text", get_text_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_get_text_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let get_value_fn = self
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.get_value", &selector);

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_value", &selector, &data) {
                    Ok(()) => wait_for_element_response(lua, rx),
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_get_value_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("get_value", get_value_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_get_value_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let get_attribute_fn = self
            .lua
            .create_function(move |lua, (selector, attr): (mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let attr = String::from_utf8_lossy(&attr.as_bytes()).into_owned();
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.element.get_attribute",
                    &format!("{} {}", selector, attr),
                );

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "attr": attr, "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_attribute", &selector, &data) {
                    Ok(()) => wait_for_element_response(lua, rx),
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_get_attribute_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table
            .set("get_attribute", get_attribute_fn)
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.set_get_attribute_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;

        let pid = plugin_id.clone();
        let get_attributes_fn = self
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ =
                    emit_permission_log(&pid, "api_call", "sl.element.get_attributes", &selector);

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_attributes", &selector, &data) {
                    Ok(()) => wait_for_element_response(lua, rx),
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_get_attributes_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table
            .set("get_attributes", get_attributes_fn)
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.set_get_attributes_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;

        let pid = plugin_id.clone();
        let click_fn = self
            .lua
            .create_function(move |_, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.click", &selector);
                let data = serde_json::json!({}).to_string();
                match emit_ui_event(&pid, "element_click", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.click_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        Ok(false)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_click_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("click", click_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_click_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let set_value_fn = self
            .lua
            .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let value = String::from_utf8_lossy(&value.as_bytes()).into_owned();
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.element.set_value",
                    &format!("{} = {}", selector, value),
                );
                let data = serde_json::json!({ "value": value }).to_string();
                match emit_ui_event(&pid, "element_set_value", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.set_value_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        Ok(false)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_set_value_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("set_value", set_value_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_set_value_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let check_fn = self
            .lua
            .create_function(move |_, (selector, checked): (mlua::String, bool)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.element.check",
                    &format!("{} = {}", selector, checked),
                );
                let data = serde_json::json!({ "checked": checked }).to_string();
                match emit_ui_event(&pid, "element_check", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.check_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        Ok(false)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_check_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("check", check_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_check_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let select_fn = self
            .lua
            .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let value = String::from_utf8_lossy(&value.as_bytes()).into_owned();
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.element.select",
                    &format!("{} = {}", selector, value),
                );
                let data = serde_json::json!({ "value": value }).to_string();
                match emit_ui_event(&pid, "element_select", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.select_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        Ok(false)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_select_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("select", select_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_select_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let focus_fn = self
            .lua
            .create_function(move |_, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.focus", &selector);
                let data = serde_json::json!({}).to_string();
                match emit_ui_event(&pid, "element_focus", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.focus_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        Ok(false)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_focus_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("focus", focus_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_focus_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let pid = plugin_id.clone();
        let blur_fn = self
            .lua
            .create_function(move |_, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.blur", &selector);
                let data = serde_json::json!({}).to_string();
                match emit_ui_event(&pid, "element_blur", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.blur_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        Ok(false)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_blur_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("blur", blur_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_blur_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        let lua_weak = self.lua.clone();
        let pid = plugin_id.clone();
        let on_change_fn = self
            .lua
            .create_function(move |_, (selector, callback): (mlua::String, mlua::Function)| {
                let selector_str = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ =
                    emit_permission_log(&pid, "api_call", "sl.element.on_change", &selector_str);

                let registry_key = format!("_element_change_callback_{}_{}", pid, selector_str);
                lua_weak
                    .set_named_registry_value(&registry_key, callback)
                    .map_err(|e| {
                        mlua::Error::runtime(i18n_service().t_with_options(
                            "element.store_callback_failed",
                            &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                        ))
                    })?;

                let cleanup_key = registry_key.clone();
                let cleanup_fn = lua_weak.create_function(move |lua, ()| {
                    lua.unset_named_registry_value(&cleanup_key).map_err(|e| {
                        mlua::Error::runtime(i18n_service().t_with_options(
                            "element.cleanup_callback_failed",
                            &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                        ))
                    })
                })?;

                let data = serde_json::json!({ "selector": selector_str }).to_string();
                match emit_ui_event(&pid, "element_on_change", &selector_str, &data) {
                    Ok(()) => Ok(cleanup_fn),
                    Err(e) => {
                        let _ = lua_weak.unset_named_registry_value(&registry_key);
                        eprintln!(
                            "[Element] {}",
                            i18n_service().t_with_options(
                                "element.on_change_error",
                                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                            )
                        );
                        let noop_fn = lua_weak.create_function(|_, ()| Ok(()))?;
                        Ok(noop_fn)
                    }
                }
            })
            .map_err(|e| {
                i18n_service().t_with_options(
                    "element.create_on_change_failed",
                    &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                )
            })?;
        element_table.set("on_change", on_change_fn).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_on_change_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        sl.set("element", element_table).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_element_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        Ok(())
    }
}
