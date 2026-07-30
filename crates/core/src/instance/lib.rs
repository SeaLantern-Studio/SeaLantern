pub mod extension;
pub mod identity;
pub mod import;
pub mod lifecycle;
pub mod model;
pub mod player;
pub mod repository;

pub use extension::{InstanceExtension, InstanceExtensionError, InstanceExtensionKind};
pub use identity::InstanceIdentity;
pub use import::{plan_import, InstanceImportError, InstanceImportPlan, InstanceImportRequest};
pub use lifecycle::{
    restart_instance, transition, InstanceLifecycleAction, InstanceLifecycleState,
    InstanceRestartDriver, LifecycleTransitionError, RestartError, RestartOutcome, RestartPolicy,
};
pub use model::{Instance, InstanceError, InstanceId, InstanceSpec, LocalLaunch, StartupMode};
pub use player::{PlayerName, PlayerNameError, PlayerSnapshot};
pub use repository::InstanceRepository;
