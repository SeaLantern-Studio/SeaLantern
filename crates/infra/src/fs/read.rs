use std::path::Path;

use tokio::io::AsyncReadExt;

use crate::observability;

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
    let result = async {
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|error| FsError::io("open file for reading", path, error))?;
        let mut bytes = Vec::new();
        file.take((limit.max_bytes as u64).saturating_add(1))
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| FsError::io("read file", path, error))?;
        if bytes.len() > limit.max_bytes {
            return Err(FsError::DataLimitExceeded {
                path: path.to_path_buf(),
                limit: limit.max_bytes,
                observed_at_least: bytes.len(),
            });
        }
        Ok(bytes)
    }
    .await;
    if let Err(error) = &result {
        observability::operation_failed("read limited file", path, error);
    }
    result
}

/// Reads UTF-8 text while enforcing a maximum returned size.
pub async fn read_string_limited(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<String, FsError> {
    let path = path.as_ref();
    let bytes = read_limited(path, limit).await?;
    let result = String::from_utf8(bytes).map_err(|error| FsError::Encoding {
        path: path.to_path_buf(),
        encoding: "UTF-8",
        message: error.to_string(),
    });
    if let Err(error) = &result {
        observability::operation_failed("decode UTF-8 text", path, error);
    }
    result
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
            Err(FsError::DataLimitExceeded { observed_at_least: 4, .. })
        ));
        std::fs::remove_dir_all(root).unwrap();
    }
}
