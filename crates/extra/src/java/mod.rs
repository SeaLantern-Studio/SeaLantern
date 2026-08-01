//! Java 环境检测适配层。

mod discovery;
mod error;
mod index;
mod mapping;
mod validation;

pub use crate::config::JavaInfo;
pub use discovery::{
    detect_java_installations, detect_java_installations_with_diagnostics,
    detect_java_installations_with_global_search, JavaDetectionReport,
};
pub use error::{JavaDiscoveryError, JavaValidationError};
pub use index::{JavaSearchDirectory, JavaSearchIndex};
pub use validation::validate_java;
