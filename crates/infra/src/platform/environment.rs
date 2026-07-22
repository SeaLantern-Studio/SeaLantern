use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::PathBuf;

/// 进程环境变量访问错误。
#[derive(Debug)]
pub enum EnvironmentError {
    InvalidKey,
    InvalidValue,
    NotUnicode { key: String },
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(formatter, "environment variable name is invalid"),
            Self::InvalidValue => write!(formatter, "environment variable value contains NUL"),
            Self::NotUnicode { key } => {
                write!(formatter, "environment variable '{key}' is not valid Unicode")
            }
        }
    }
}

impl std::error::Error for EnvironmentError {}

/// 进程环境变量的受控访问入口。
pub struct Environment;

impl Environment {
    /// 返回原始平台字符串，避免丢失非 UTF-8 环境变量。
    pub fn get_os(key: impl AsRef<OsStr>) -> Option<OsString> {
        env::var_os(key)
    }

    /// 读取 UTF-8 环境变量；变量不存在时返回 `Ok(None)`。
    pub fn get(key: &str) -> Result<Option<String>, EnvironmentError> {
        match env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => {
                Err(EnvironmentError::NotUnicode { key: key.into() })
            }
        }
    }

    /// 设置进程环境变量。
    ///
    /// 该操作影响整个进程。调用方必须在启动线程或加载不受控库之前完成写入，
    /// 避免与并发环境读取产生未定义的外部行为。
    pub fn set(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Result<(), EnvironmentError> {
        validate_key(key.as_ref())?;
        validate_value(value.as_ref())?;
        env::set_var(key, value);
        Ok(())
    }

    /// 删除进程环境变量。与 [`Self::set`] 一样，必须由调用方保证没有并发访问。
    pub fn remove(key: impl AsRef<OsStr>) -> Result<(), EnvironmentError> {
        validate_key(key.as_ref())?;
        env::remove_var(key);
        Ok(())
    }

    /// 返回 `PATH` 的已解析条目；未设置时返回空列表。
    pub fn path_entries() -> Vec<PathBuf> {
        env::var_os("PATH")
            .map(|value| env::split_paths(&value).collect())
            .unwrap_or_default()
    }

    /// 设置 `PATH` 的全部条目。
    pub fn set_path(entries: impl IntoIterator<Item = PathBuf>) -> Result<(), EnvironmentError> {
        Self::set("PATH", env::join_paths(entries).map_err(|_| EnvironmentError::InvalidValue)?)
    }
}

fn validate_key(key: &OsStr) -> Result<(), EnvironmentError> {
    let key = key.to_string_lossy();
    if key.is_empty() || key.contains('=') || key.contains('\0') {
        return Err(EnvironmentError::InvalidKey);
    }
    Ok(())
}

fn validate_value(value: &OsStr) -> Result<(), EnvironmentError> {
    if value.to_string_lossy().contains('\0') {
        return Err(EnvironmentError::InvalidValue);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;

    fn environment_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn reads_writes_and_removes_a_utf8_value() {
        let _guard = environment_lock().lock().unwrap();
        let key = "SEALANTERN_INFRA_PLATFORM_ENVIRONMENT_TEST";
        let previous = Environment::get_os(key);

        Environment::set(key, "value").unwrap();
        assert_eq!(Environment::get(key).unwrap(), Some("value".into()));
        Environment::remove(key).unwrap();
        assert_eq!(Environment::get(key).unwrap(), None);

        if let Some(previous) = previous {
            Environment::set(key, previous).unwrap();
        }
    }

    #[test]
    fn rejects_invalid_variable_names() {
        assert!(matches!(Environment::set("", "value"), Err(EnvironmentError::InvalidKey)));
        assert!(matches!(Environment::remove("A=B"), Err(EnvironmentError::InvalidKey)));
    }
}
