use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const SERVER_METADATA_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// 已完成服务端检查后保存的轻量摘要，不包含证据明细。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataSnapshot {
    pub schema_version: u16,
    pub inspected_at_unix_secs: u64,
    pub subject: ServerMetadataSubject,
    pub identity: Option<ServerMetadataIdentity>,
    pub minecraft: Option<ServerMetadataMinecraft>,
    pub java: ServerMetadataJava,
    pub components: Vec<ServerMetadataComponent>,
    pub launches: Vec<ServerMetadataLaunch>,
    pub diagnostics: Vec<ServerMetadataDiagnostic>,
}

impl ServerMetadataSnapshot {
    pub fn validity_for(
        &self,
        fingerprint: Option<&ServerMetadataFingerprint>,
    ) -> ServerMetadataSnapshotValidity {
        if self.schema_version != SERVER_METADATA_SNAPSHOT_SCHEMA_VERSION {
            return ServerMetadataSnapshotValidity::SchemaMismatch;
        }
        let Some(saved) = self.subject.fingerprint.as_ref() else {
            return ServerMetadataSnapshotValidity::MissingFingerprint;
        };
        let Some(current) = fingerprint else {
            return ServerMetadataSnapshotValidity::MissingFingerprint;
        };
        if saved == current {
            ServerMetadataSnapshotValidity::Current
        } else {
            ServerMetadataSnapshotValidity::FingerprintChanged
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataSubject {
    pub kind: ServerMetadataSubjectKind,
    pub size_bytes: Option<u64>,
    pub modified_at_unix_secs: Option<u64>,
    pub fingerprint: Option<ServerMetadataFingerprint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerMetadataSubjectKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataFingerprint {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataIdentity {
    pub category: String,
    pub implementation_key: String,
    pub implementation_name: String,
    pub implementation_confidence: u8,
    pub version: Option<String>,
    pub version_confidence: u8,
    pub release_channel: Option<String>,
    pub ecosystems: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataMinecraft {
    pub version: Option<String>,
    pub version_confidence: u8,
    pub id: Option<String>,
    pub name: Option<String>,
    pub java_version: Option<u16>,
    pub stable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServerMetadataJava {
    pub required_major: Option<u16>,
    pub required_major_confidence: u8,
    pub runtime_component: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataComponent {
    pub kind: String,
    pub key: String,
    pub name: String,
    pub version: Option<String>,
    pub confidence: u8,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataLaunch {
    pub id: String,
    pub platform: String,
    pub target_kind: String,
    pub target_path: Option<PathBuf>,
    pub confidence: u8,
    pub required_java_major: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerMetadataDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMetadataSnapshotValidity {
    Current,
    FingerprintChanged,
    MissingFingerprint,
    SchemaMismatch,
}

#[cfg(test)]
mod tests {
    use super::{
        ServerMetadataFingerprint, ServerMetadataSnapshot, ServerMetadataSnapshotValidity,
        ServerMetadataSubject, ServerMetadataSubjectKind, SERVER_METADATA_SNAPSHOT_SCHEMA_VERSION,
    };

    fn snapshot(value: &str) -> ServerMetadataSnapshot {
        ServerMetadataSnapshot {
            schema_version: SERVER_METADATA_SNAPSHOT_SCHEMA_VERSION,
            inspected_at_unix_secs: 1,
            subject: ServerMetadataSubject {
                kind: ServerMetadataSubjectKind::File,
                size_bytes: Some(1),
                modified_at_unix_secs: Some(1),
                fingerprint: Some(ServerMetadataFingerprint {
                    algorithm: "sha256".to_string(),
                    value: value.to_string(),
                }),
            },
            identity: None,
            minecraft: None,
            java: Default::default(),
            components: Vec::new(),
            launches: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn fingerprint_validity_distinguishes_current_and_changed_artifacts() {
        let current = ServerMetadataFingerprint {
            algorithm: "sha256".to_string(),
            value: "abc".to_string(),
        };
        let snapshot = snapshot("abc");

        assert_eq!(snapshot.validity_for(Some(&current)), ServerMetadataSnapshotValidity::Current);
        assert_eq!(
            snapshot.validity_for(Some(&ServerMetadataFingerprint {
                algorithm: "sha256".to_string(),
                value: "def".to_string(),
            })),
            ServerMetadataSnapshotValidity::FingerprintChanged
        );
        assert_eq!(snapshot.validity_for(None), ServerMetadataSnapshotValidity::MissingFingerprint);
    }

    #[test]
    fn serialized_snapshot_does_not_contain_evidence_details() {
        let json = serde_json::to_string(&snapshot("abc")).expect("serialize snapshot");
        assert!(!json.contains("evidence"));
    }
}
