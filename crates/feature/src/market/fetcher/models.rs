//! 资源获取器（Fetcher）模块的数据模型。
//!
//! 定义了从资源平台获取到的文件级数据结构，如 [`VersionFile`]，用于描述
//! 某个版本关联的具体文件信息（下载链接、文件名、大小等）。

use serde::{Deserialize, Serialize};

/// 版本关联的文件信息。
///
/// 每个版本可能包含一个或多个文件（例如主 jar 包、API jar 包等），
/// 该结构描述了其中单个文件的下载地址、名称、大小以及是否为默认下载文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    /// 文件的下载 URL。
    pub url: String,

    /// 文件名（不含路径）。
    pub filename: String,

    /// 文件大小，单位为字节。
    pub size: u64,

    /// 是否为该版本的主文件（默认下载项）。
    ///
    /// 当同一版本包含多个文件时，`primary = true` 表示该文件是用户
    /// 通常应下载的那个（如插件本体）。
    pub primary: bool,
}
