pub(super) fn emit_process_log(plugin_id: &str, action: &str, detail: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", action, detail);
}

pub(super) fn process_error(prefix: &str, error: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::runtime(format!("{}: {}", prefix, error))
}

pub(super) fn mask_args_for_log(args: &[String]) -> String {
    if args.is_empty() {
        "[]".to_string()
    } else {
        format!("[{} args]", args.len())
    }
}

pub(super) fn validate_program_path(
    program_path: &std::path::Path,
    program: &str,
) -> Result<(), mlua::Error> {
    if !program_path.exists() {
        return Err(mlua::Error::runtime(format!("Program not found: {}", program)));
    }

    if !program_path.is_file() {
        return Err(mlua::Error::runtime(format!("Program path is not a file: {}", program)));
    }

    Ok(())
}

pub(super) fn validate_args(
    args: &[String],
    max_args_count: usize,
    max_arg_length: usize,
) -> Result<(), mlua::Error> {
    if args.len() > max_args_count {
        return Err(mlua::Error::runtime(format!(
            "Too many arguments: maximum {} allowed",
            max_args_count
        )));
    }

    for arg in args {
        if arg.len() > max_arg_length {
            return Err(mlua::Error::runtime(format!(
                "Argument too long: maximum {} characters allowed",
                max_arg_length
            )));
        }
    }

    Ok(())
}

fn is_allowed_env_key(key: &str) -> bool {
    !matches!(
        key.to_ascii_uppercase().as_str(),
        "PATH"
            | "PATHEXT"
            | "LD_PRELOAD"
            | "LD_LIBRARY_PATH"
            | "DYLD_INSERT_LIBRARIES"
            | "DYLD_LIBRARY_PATH"
            | "SYSTEMROOT"
            | "COMSPEC"
            | "PROMPT"
            | "PSMODULEPATH"
    )
}

pub(super) fn collect_env_vars(
    env_table: mlua::Table,
    max_env_vars: usize,
    max_env_key_length: usize,
    max_env_value_length: usize,
) -> Result<Vec<(String, String)>, mlua::Error> {
    let mut env_vars = Vec::new();

    for pair in env_table.pairs::<String, String>() {
        let (k, v) = pair?;

        if env_vars.len() >= max_env_vars {
            return Err(mlua::Error::runtime(format!(
                "Too many environment variables: maximum {} allowed",
                max_env_vars
            )));
        }

        if k.is_empty() || k.len() > max_env_key_length {
            return Err(mlua::Error::runtime(format!(
                "Invalid environment key length: maximum {} characters allowed",
                max_env_key_length
            )));
        }

        if v.len() > max_env_value_length {
            return Err(mlua::Error::runtime(format!(
                "Environment value too long: maximum {} characters allowed",
                max_env_value_length
            )));
        }

        if !is_allowed_env_key(&k) {
            return Err(mlua::Error::runtime(format!(
                "Environment variable is not allowed: {}",
                k
            )));
        }

        env_vars.push((k, v));
    }

    Ok(env_vars)
}
