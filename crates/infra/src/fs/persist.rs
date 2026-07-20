use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{read_limited, write_atomic, DataLimit, FsError};

/// Reads a JSON file within a maximum byte size.
pub async fn read_json<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<T, FsError> {
    let bytes = read_limited(path, limit).await?;
    serde_json::from_slice(&bytes).map_err(|error| FsError::Serialization(error.to_string()))
}

/// Serializes and atomically writes a JSON file.
pub async fn write_json_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), FsError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| FsError::Serialization(error.to_string()))?;
    write_atomic(path, &bytes).await
}

/// Reads a TOML file within a maximum byte size.
pub async fn read_toml<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<T, FsError> {
    let text = String::from_utf8(read_limited(path, limit).await?)
        .map_err(|error| FsError::Serialization(error.to_string()))?;
    toml::from_str(&text).map_err(|error| FsError::Serialization(error.to_string()))
}

/// Serializes and atomically writes a TOML file.
pub async fn write_toml_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), FsError> {
    let text =
        toml::to_string_pretty(value).map_err(|error| FsError::Serialization(error.to_string()))?;
    write_atomic(path, text.as_bytes()).await
}

/// Reads a YAML file within a maximum byte size.
pub async fn read_yaml<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    limit: DataLimit,
) -> Result<T, FsError> {
    let bytes = read_limited(path, limit).await?;
    serde_yaml::from_slice(&bytes).map_err(|error| FsError::Serialization(error.to_string()))
}

/// Serializes and atomically writes a YAML file.
pub async fn write_yaml_atomic<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> Result<(), FsError> {
    let text =
        serde_yaml::to_string(value).map_err(|error| FsError::Serialization(error.to_string()))?;
    write_atomic(path, text.as_bytes()).await
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
        std::fs::remove_dir_all(root).unwrap();
    }
}
