//! Shared server-configuration helpers for startup files, properties, and discovery.

pub mod cpu_policy;
pub mod discovery;
pub mod properties;
pub mod startup;
pub mod types;

pub use cpu_policy::{
    format_cpu_range, parse_cpu_set, resolve_active_processor_count, resolve_cpu_policy,
    resolve_local_cpu_policy, resolve_unbounded_active_processor_count, ResolvedCpuPolicy,
};
pub use discovery::{
    discover_server_config_files, discover_server_config_files_in_dir,
    discover_server_config_files_in_dir_with_options, discover_server_config_files_with_options,
    resolve_discovered_config_path, resolve_discovered_config_path_with_options,
    resolve_primary_port_config_path, resolve_primary_server_properties_path,
    resolve_primary_startup_config_path, search_server_config_files,
    search_server_config_files_in_entries, search_server_config_files_with_options,
    SUPPORTED_SERVER_CONFIG_EXTENSIONS,
};
pub use startup::{
    build_managed_jvm_args_from_input, resolve_effective_startup_config_from_document,
    EffectiveStartupConfig, ManagedJvmBuildInput, StartupResolutionDefaults,
    StartupRuntimeDefaults,
};
