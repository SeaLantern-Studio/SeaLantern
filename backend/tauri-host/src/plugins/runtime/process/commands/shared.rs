pub(super) fn emit_process_log(plugin_id: &str, action: &str, detail: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", action, detail);
}

use crate::plugins::runtime::process::common::process_err1;

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
        return Err(process_err1("plugins.runtime.process.program_not_found", program));
    }

    if !program_path.is_file() {
        return Err(process_err1("plugins.runtime.process.program_path_not_file", program));
    }

    Ok(())
}

pub(super) fn validate_args(
    args: &[String],
    max_args_count: usize,
    max_arg_length: usize,
) -> Result<(), mlua::Error> {
    if args.len() > max_args_count {
        return Err(process_err1(
            "plugins.runtime.process.too_many_arguments",
            max_args_count.to_string(),
        ));
    }

    for arg in args {
        if arg.len() > max_arg_length {
            return Err(process_err1(
                "plugins.runtime.process.argument_too_long",
                max_arg_length.to_string(),
            ));
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
            return Err(process_err1(
                "plugins.runtime.process.too_many_env_vars",
                max_env_vars.to_string(),
            ));
        }

        if k.is_empty() || k.len() > max_env_key_length {
            return Err(process_err1(
                "plugins.runtime.process.env_key_length_invalid",
                max_env_key_length.to_string(),
            ));
        }

        if v.len() > max_env_value_length {
            return Err(process_err1(
                "plugins.runtime.process.env_value_too_long",
                max_env_value_length.to_string(),
            ));
        }

        if !is_allowed_env_key(&k) {
            return Err(process_err1("plugins.runtime.process.env_var_not_allowed", k));
        }

        env_vars.push((k, v));
    }

    Ok(env_vars)
}
