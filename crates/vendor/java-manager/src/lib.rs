//! A library for locating Java installations on the local system.
//!
//! This crate provides functionality to:
//! - Discover Java runtimes via `PATH`, `JAVA_HOME`, or deep system scans.
//! - Extract detailed metadata (version, vendor, architecture) from each installation.
//!
//! # Examples
//!
//! ```no_run
//! use java_manager::java_home;
//!
//! // Find all Java installations in PATH
//! let java = java_home().unwrap();
//! println!("Java version: {}", java.version);
//! # Ok::<_, java_manager::JavaError>(())
//! ```

pub mod error;
pub mod global_search;
pub mod info;
pub mod local;
pub mod search;

pub use error::JavaError;
pub use global_search::environment_search;
pub use global_search::environment_search_with_diagnostics;
pub use global_search::global_search;
pub use global_search::global_search_complete;
pub use global_search::global_search_with_index;
pub use info::JavaInfo;
pub use local::java_home;
pub use local::java_home_with_diagnostics;
pub use search::GlobalSearchDirectory;
pub use search::GlobalSearchIndex;
pub use search::GlobalSearchOptions;
pub use search::SearchError;
pub use search::SearchReport;
pub use search::deep_search;
pub use search::deep_search_with_diagnostics;
pub use search::full_search;
pub use search::full_search_with_diagnostics;
pub use search::quick_search;
pub use search::quick_search_with_diagnostics;
