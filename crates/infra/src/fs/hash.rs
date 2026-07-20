use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

use super::FsError;

/// Calculates a SHA-256 digest for a file without loading it fully into memory.
pub async fn sha256_file(path: impl AsRef<Path>) -> Result<[u8; 32], FsError> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

/// Calculates a lowercase hexadecimal SHA-256 digest for in-memory data.
pub fn sha256_hex(data: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(data.as_ref());
    format!("{digest:x}")
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
}
