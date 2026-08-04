use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{read_limited, write_atomic, DataLimit, FsError};

/// 在最大字节大小限制内读取 JSON 文件。
pub async fn read_json<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<T, FsError> {
    let path = path.as_ref();
    let bytes = read_limited(path, limit).await?;
    serde_json::from_slice(&bytes).map_err(|error| codec_error("JSON", "decode", path, error))
}

/// 序列化并以原子方式写入 JSON 文件。
pub async fn write_json_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), FsError> {
    let path = path.as_ref();
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| codec_error("JSON", "encode", path, error))?;
    write_atomic(path, &bytes).await
}

/// 在最大字节大小限制内读取 TOML 文件。
pub async fn read_toml<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<T, FsError> {
    let path = path.as_ref();
    let text = String::from_utf8(read_limited(path, limit).await?)
        .map_err(|error| codec_error("TOML", "decode UTF-8", path, error))?;
    toml::from_str(&text).map_err(|error| codec_error("TOML", "decode", path, error))
}

/// 序列化并以原子方式写入 TOML 文件。
pub async fn write_toml_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), FsError> {
    let path = path.as_ref();
    let text = toml::to_string_pretty(value)
        .map_err(|error| codec_error("TOML", "encode", path, error))?;
    write_atomic(path, text.as_bytes()).await
}

/// 在最大字节大小限制内读取 YAML 文件。
pub async fn read_yaml<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<T, FsError> {
    let path = path.as_ref();
    let bytes = read_limited(path, limit).await?;
    serde_yaml::from_slice(&bytes).map_err(|error| codec_error("YAML", "decode", path, error))
}

/// 序列化并以原子方式写入 YAML 文件。
pub async fn write_yaml_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), FsError> {
    let path = path.as_ref();
    let text =
        serde_yaml::to_string(value).map_err(|error| codec_error("YAML", "encode", path, error))?;
    write_atomic(path, text.as_bytes()).await
}

fn codec_error(
    format: &'static str,
    operation: &'static str,
    path: &Path,
    error: impl std::fmt::Display,
) -> FsError {
    let error = FsError::serialization(format, operation, path, error.to_string());
    crate::observability::serialization_failed(format, operation, path, &error);
    error
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct Settings {
        enabled: bool,
        port: u16,
    }

    #[tokio::test]
    async fn persists_json_toml_and_yaml() {
        let root = crate::fs::test_dir("persist");
        let settings = Settings { enabled: true, port: 25565 };
        let limit = DataLimit::new(1024);

        let json = root.join("settings.json");
        write_json_atomic(&json, &settings).await.unwrap();
        assert_eq!(read_json::<Settings>(&json, limit).await.unwrap(), settings);

        let toml = root.join("settings.toml");
        write_toml_atomic(&toml, &settings).await.unwrap();
        assert_eq!(read_toml::<Settings>(&toml, limit).await.unwrap(), settings);

        let yaml = root.join("settings.yaml");
        write_yaml_atomic(&yaml, &settings).await.unwrap();
        assert_eq!(read_yaml::<Settings>(&yaml, limit).await.unwrap(), settings);

        let invalid_json = root.join("invalid.json");
        tokio::fs::write(&invalid_json, b"{").await.unwrap();
        assert!(matches!(
            read_json::<Settings>(&invalid_json, limit).await,
            Err(FsError::Serialization { format: "JSON", operation: "decode", .. })
        ));
        std::fs::remove_dir_all(root).unwrap();
    }
}
