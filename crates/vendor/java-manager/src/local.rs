//! Functions for accessing Java from the local environment.

use crate::JavaInfo;
use std::env;

/// Returns a `JavaInfo` for the Java installation pointed to by the `JAVA_HOME`
/// environment variable, if set and valid.
///
/// This function reads the `JAVA_HOME` variable, treats it as a path, and attempts
/// to create a `JavaInfo` from it. If the variable is not set or if the resulting
/// `JavaInfo` cannot be created (e.g., the path does not contain a valid Java),
/// `None` is returned.
///
/// # Examples
///
/// ```no_run
/// use java_manager::java_home;
///
/// if let Some(java) = java_home() {
///     println!("JAVA_HOME points to Java version {}", java.version);
/// } else {
///     println!("JAVA_HOME is not set or invalid");
/// }
/// ```
pub fn java_home() -> Option<JavaInfo> {
    java_home_with_diagnostics().ok().flatten()
}

/// Returns JAVA_HOME discovery errors instead of silently discarding them.
pub fn java_home_with_diagnostics() -> Result<Option<JavaInfo>, crate::JavaError> {
    let Some(path) = env::var_os("JAVA_HOME") else {
        return Ok(None);
    };

    JavaInfo::from_discovered_path(path.to_string_lossy().into_owned()).map(Some)
}
