use std::path::PathBuf;

use java_manager::{GlobalSearchDirectory, GlobalSearchIndex};
use serde::{Deserialize, Serialize};

pub(crate) const JAVA_SEARCH_INDEX_VERSION: u32 = 2;

/// 可由上游调用方持久化的 Java 全局搜索索引。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JavaSearchIndex {
    pub schema_version: u32,
    pub roots: Vec<String>,
    pub directories: Vec<JavaSearchDirectory>,
    pub candidates: Vec<String>,
    pub max_depth: Option<usize>,
    pub max_directories: usize,
    pub truncated: bool,
}

impl Default for JavaSearchIndex {
    fn default() -> Self {
        Self {
            schema_version: JAVA_SEARCH_INDEX_VERSION,
            roots: Vec::new(),
            directories: Vec::new(),
            candidates: Vec::new(),
            max_depth: None,
            max_directories: usize::MAX,
            truncated: false,
        }
    }
}

/// 全局搜索索引中的目录元数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JavaSearchDirectory {
    pub path: String,
    pub modified_ns: u64,
    pub len: u64,
}

pub(crate) fn to_vendor_search_index(index: &JavaSearchIndex) -> GlobalSearchIndex {
    GlobalSearchIndex {
        roots: index.roots.iter().map(PathBuf::from).collect(),
        directories: index
            .directories
            .iter()
            .map(|directory| GlobalSearchDirectory {
                path: PathBuf::from(&directory.path),
                modified_ns: directory.modified_ns,
                len: directory.len,
            })
            .collect(),
        candidates: index.candidates.iter().map(PathBuf::from).collect(),
        max_depth: index.max_depth,
        max_directories: index.max_directories,
        truncated: index.truncated,
    }
}

pub(crate) fn from_vendor_search_index(index: GlobalSearchIndex) -> JavaSearchIndex {
    JavaSearchIndex {
        schema_version: JAVA_SEARCH_INDEX_VERSION,
        roots: index
            .roots
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        directories: index
            .directories
            .iter()
            .map(|directory| JavaSearchDirectory {
                path: directory.path.to_string_lossy().into_owned(),
                modified_ns: directory.modified_ns,
                len: directory.len,
            })
            .collect(),
        candidates: index
            .candidates
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        max_depth: index.max_depth,
        max_directories: index.max_directories,
        truncated: index.truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::{JavaSearchDirectory, JavaSearchIndex};

    #[test]
    fn search_index_is_serializable_for_upstream_storage() {
        let index = JavaSearchIndex {
            roots: vec!["/opt".to_string()],
            directories: vec![JavaSearchDirectory {
                path: "/opt/jdk".to_string(),
                modified_ns: 10,
                len: 20,
            }],
            candidates: vec!["/opt/jdk/bin/java".to_string()],
            ..JavaSearchIndex::default()
        };

        let encoded = serde_json::to_string(&index).expect("search index should serialize");
        let decoded: JavaSearchIndex =
            serde_json::from_str(&encoded).expect("search index should deserialize");

        assert_eq!(decoded, index);
    }
}
