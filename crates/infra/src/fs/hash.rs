//! 哈希计算与校验和解析工具。
//!
//! 提供 SHA-256 文件摘要计算、内存数据快速哈希、校验和文件内容解析等功能。
//!
//! # 功能概览
//!
//! | 函数 | 用途 |
//! |------|------|
//! | [`sha256_file`] | 异步流式计算文件的 SHA-256 摘要，内存占用恒定 |
//! | [`sha256_hex`] | 计算内存数据的小写十六进制 SHA-256 |
//! | [`is_sha256_hex`] | 检查字符串是否为有效的 64 位十六进制哈希值 |
//! | [`find_sha256_in_line`] | 在文本行中查找 SHA-256 哈希值 |
//! | [`parse_sha256_from_checksum_content`] | 从校验和文件内容中解析目标文件的哈希值 |

use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

use crate::observability;

use super::FsError;

/// 计算文件的 SHA-256 摘要，无需将整个文件加载到内存中。
pub async fn sha256_file(path: impl AsRef<Path>) -> Result<[u8; 32], FsError> {
    let path = path.as_ref();
    let result = async {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|error| FsError::io("open file for hashing", path, error))?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .await
                .map_err(|error| FsError::io("read file for hashing", path, error))?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        Ok(digest.finalize().into())
    }
    .await;
    if let Err(error) = &result {
        observability::operation_failed("calculate SHA-256", path, error);
    }
    result
}

/// 计算内存数据的小写十六进制 SHA-256 摘要。
pub fn sha256_hex(data: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(data.as_ref());
    format!("{digest:x}")
}

/// 检查字符串是否为有效的 SHA256 十六进制值（64 位小写 hex）。
pub fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

/// 在文本行中查找 SHA256 哈希值，返回标准化后的小写形式。
///
/// 以空白字符或 `=` `:` `,` `;` `(` `)` `[` `]` `{` `}` `<` `>` 分隔 token，
/// 并去除 `*` `"` `'` 等前缀/包裹字符。
pub fn find_sha256_in_line(line: &str) -> Option<String> {
    for token in line.split(|ch: char| {
        ch.is_ascii_whitespace()
            || matches!(ch, '=' | ':' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
    }) {
        let candidate = token.trim_matches(|ch| ch == '*' || ch == '"' || ch == '\'');
        if is_sha256_hex(candidate) {
            return Some(candidate.to_ascii_lowercase());
        }
    }

    None
}

/// 从校验和文件内容中解析目标文件的 SHA256 哈希值。
///
/// 支持标准校验和文件格式（每行 `<hash>  <filename>` 或 `*<hash>  <filename>`）。
///
/// # 解析逻辑
///
/// - 如果内容中只有一行哈希值，无论是否匹配目标名都返回该哈希。
/// - 如果存在多行，则按文件名匹配（不区分大小写）。
/// - 若未匹配到任何行，返回 `None`。
pub fn parse_sha256_from_checksum_content(content: &str, target_name: &str) -> Option<String> {
    let target_lower = target_name.to_ascii_lowercase();
    let target_file_name = Path::new(target_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(target_name)
        .to_ascii_lowercase();

    let mut single_hash: Option<String> = None;
    let mut hash_line_count = 0_usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let hash = match find_sha256_in_line(trimmed) {
            Some(value) => value,
            None => continue,
        };

        hash_line_count += 1;
        if hash_line_count == 1 {
            single_hash = Some(hash.clone());
        } else {
            single_hash = None;
        }

        let line_lower = trimmed.to_ascii_lowercase();
        if line_lower.contains(&target_lower) || line_lower.contains(&target_file_name) {
            return Some(hash);
        }
    }

    if hash_line_count == 1 {
        return single_hash;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_known_value() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn is_sha256_hex_validates_length_and_chars() {
        assert!(is_sha256_hex(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ));
        assert!(!is_sha256_hex("short"));
        assert!(!is_sha256_hex(
            "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg"
        )); // 'g' not hex
    }

    #[test]
    fn find_sha256_in_line_finds_valid_hash() {
        let line = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  file.zip";
        let result = find_sha256_in_line(line);
        assert_eq!(
            result.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }

    #[test]
    fn find_sha256_in_line_handles_star_prefix() {
        let line = "*e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 file.zip";
        let result = find_sha256_in_line(line);
        assert_eq!(
            result.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }

    #[test]
    fn parse_sha256_from_single_line_matches_target() {
        let content =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  file.zip\n";
        let result = parse_sha256_from_checksum_content(content, "file.zip");
        assert_eq!(
            result.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }

    #[test]
    fn parse_sha256_from_multi_line_returns_target_match() {
        let content = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  other.zip
a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a  target.zip
";
        let result = parse_sha256_from_checksum_content(content, "target.zip");
        assert_eq!(
            result.as_deref(),
            Some("a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a")
        );
    }

    #[test]
    fn parse_sha256_returns_single_hash_when_no_target_match() {
        let content =
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a  only_one.zip\n";
        let result = parse_sha256_from_checksum_content(content, "other.zip");
        assert_eq!(
            result.as_deref(),
            Some("a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a")
        );
    }

    #[test]
    fn parse_sha256_returns_none_when_no_hash_found() {
        let content = "not a hash file content\n";
        let result = parse_sha256_from_checksum_content(content, "file.zip");
        assert!(result.is_none());
    }

    #[test]
    fn parse_sha256_returns_none_for_empty_content() {
        let result = parse_sha256_from_checksum_content("", "file.zip");
        assert!(result.is_none());
    }
}
