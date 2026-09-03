pub mod extension;
pub mod identity;
pub mod import;
pub mod lifecycle;
pub mod model;
pub mod player;
pub mod repository;
pub mod server_metadata;

pub use extension::{InstanceExtension, InstanceExtensionError, InstanceExtensionKind};
pub use identity::InstanceIdentity;
pub use import::{InstanceImportError, InstanceImportPlan, InstanceImportRequest, plan_import};
pub use lifecycle::{
    InstanceLifecycleAction, InstanceLifecycleState, InstanceRestartDriver,
    LifecycleTransitionError, RestartError, RestartOutcome, RestartPolicy, restart_instance,
    transition,
};
pub use model::{Instance, InstanceError, InstanceId, InstanceSpec, LocalLaunch, StartupMode};
pub use player::{PlayerName, PlayerNameError, PlayerSnapshot};
pub use repository::InstanceRepository;
pub use server_metadata::{
    SERVER_METADATA_SNAPSHOT_SCHEMA_VERSION, ServerMetadataComponent, ServerMetadataDiagnostic,
    ServerMetadataFingerprint, ServerMetadataIdentity, ServerMetadataJava, ServerMetadataLaunch,
    ServerMetadataMinecraft, ServerMetadataSnapshot, ServerMetadataSnapshotValidity,
    ServerMetadataSubject, ServerMetadataSubjectKind,
};
