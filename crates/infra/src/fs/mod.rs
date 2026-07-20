//! File system infrastructure primitives.
//!
//! This module deliberately keeps policy out of the infrastructure layer. It
//! provides safe path handling, bounded reads, durable writes, persistence
//! formats, and maintenance helpers that higher layers can compose.

mod archive;
mod atomic;
mod cache;
mod cleanup;
mod dir;
mod error;
mod hash;
mod lock;
mod metadata;
mod path;
mod persist;
mod read;

pub use archive::{archive_entry_destination, parse_symbolic_link_target};
pub use atomic::write_atomic;
pub use cache::FileCache;
pub use cleanup::{clear_directory, remove_if_exists};
pub use dir::{ensure_dir, ensure_parent};
pub use error::FsError;
pub use hash::{sha256_file, sha256_hex};
pub use lock::FileLock;
pub use metadata::{describe, file_size, FileMetadata};
pub use path::SafeRelativePath;
pub use persist::{
    read_json, read_toml, read_yaml, write_json_atomic, write_toml_atomic, write_yaml_atomic,
};
pub use read::{read_limited, read_string_limited, DataLimit};

#[cfg(test)]
pub(crate) fn test_dir(label: &str) -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("sealantern-infra-fs-{label}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&path).expect("create test directory");
    path
}
