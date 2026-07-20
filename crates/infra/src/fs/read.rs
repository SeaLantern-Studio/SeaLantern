use std::path::Path;

use tokio::io::AsyncReadExt;

use super::FsError;

/// The maximum number of bytes a bounded read may return.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataLimit {
    max_bytes: usize,
}

impl DataLimit {
    /// Creates a byte limit.
    pub const fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }

    /// Returns the maximum number of bytes permitted.
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}

/// Reads a file while enforcing a maximum returned size.
pub async fn read_limited(path: impl AsRef<Path>, limit: DataLimit) -> Result<Vec<u8>, FsError> {
    let path = path.as_ref();
    let file = tokio::fs::File::open(path).await?;
    let mut bytes = Vec::new();
    file.take((limit.max_bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() > limit.max_bytes {
        return Err(FsError::DataLimitExceeded {
            path: path.to_path_buf(),
            limit: limit.max_bytes,
        });
    }
    Ok(bytes)
}

/// Reads UTF-8 text while enforcing a maximum returned size.
pub async fn read_string_limited(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<String, FsError> {
    let bytes = read_limited(path, limit).await?;
    String::from_utf8(bytes).map_err(|error| FsError::Serialization(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_oversized_file() {
        let root = crate::fs::test_dir("limit");
        let file = root.join("payload.bin");
        tokio::fs::write(&file, b"1234").await.unwrap();

        assert!(matches!(
            read_limited(&file, DataLimit::new(3)).await,
            Err(FsError::DataLimitExceeded { .. })
        ));
        std::fs::remove_dir_all(root).unwrap();
    }
}
