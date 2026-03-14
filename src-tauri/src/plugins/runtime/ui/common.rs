// 公共助手函数

use crate::utils::logger::log_error;

pub(super) fn lua_str(s: mlua::String) -> String {
    String::from_utf8_lossy(&s.as_bytes()).into_owned()
}

const REG_PREFIX: &str = "_ui_error_mode_";

pub(super) fn set_error_mode(lua: &mlua::Lua, pid: &str, mode: &str) -> mlua::Result<()> {
    let mode = match mode {
        "compat" | "strict" => mode,
        other => {
            return Err(mlua::Error::runtime(format!(
                "无效的错误模式: {}（仅支持 'compat' 或 'strict'）",
                other
            )))
        }
    };
    let key = format!("{}{}", REG_PREFIX, pid);
    lua.set_named_registry_value(&key, mode.to_string())
}

fn get_error_mode(lua: &mlua::Lua, pid: &str) -> String {
    let key = format!("{}{}", REG_PREFIX, pid);
    lua.named_registry_value::<String>(&key)
        .unwrap_or_else(|_| "compat".to_string())
}

pub(super) fn emit_result(
    lua: &mlua::Lua,
    pid: &str,
    ctx: &str,
    result: Result<(), String>,
) -> mlua::Result<bool> {
    match result {
        Ok(()) => Ok(true),
        Err(e) => {
            log_error(&format!("[UI] {} 错误: {}", ctx, e));
            if get_error_mode(lua, pid) == "strict" {
                Err(mlua::Error::runtime(format!("UI {} 失败: {}", ctx, e)))
            } else {
                Ok(false)
            }
        }
    }
}

pub(super) fn map_create_err<T>(res: mlua::Result<T>, fullname: &str) -> Result<T, String> {
    res.map_err(|e| format!("创建 {} 失败: {}", fullname, e))
}

pub(super) fn map_set_err(res: mlua::Result<()>, fullname: &str) -> Result<(), String> {
    res.map_err(|e| format!("设置 {} 失败: {}", fullname, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_lua_str() {
        let lua = Lua::new();
        let s = lua.create_string(b"hello").unwrap();
        assert_eq!(lua_str(s), "hello".to_string());
    }

    #[test]
    fn test_emit_result_compat() {
        let lua = Lua::new();
        set_error_mode(&lua, "pid1", "compat").unwrap();
        let ok = emit_result(&lua, "pid1", "ctx", Err("boom".to_string())).unwrap();
        assert!(!ok);
    }

    #[test]
    fn test_emit_result_strict() {
        let lua = Lua::new();
        set_error_mode(&lua, "pid2", "strict").unwrap();
        let res = emit_result(&lua, "pid2", "ctx", Err("boom".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_set_error_mode_invalid() {
        let lua = Lua::new();
        let res = set_error_mode(&lua, "pid3", "bad-mode");
        assert!(res.is_err());
    }
}
