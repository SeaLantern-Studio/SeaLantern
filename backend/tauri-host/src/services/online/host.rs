use crate::models::settings::OneBot11Settings;
#[cfg(feature = "online-tunnel")]
use crate::services::global::i18n_service;
use crate::services::global::settings_manager;
use crate::utils::logger::{log_info_ctx, log_warn_ctx};
#[cfg(feature = "online-tunnel")]
use std::collections::HashMap;
#[cfg(feature = "online-tunnel")]
use std::path::PathBuf;

/// Host-only dependency boundary for the online domain.
///
/// Future crate extraction should depend on this module shape instead of
/// reaching into global singletons, logging, or app-data resolution directly.
pub(super) fn current_onebot_settings() -> OneBot11Settings {
    settings_manager().get().onebot_11
}

pub(super) fn log_onebot_info(action: &str, message: &str) {
    log_info_ctx("services.online.onebot", action, message);
}

pub(super) fn log_onebot_warn(action: &str, message: &str) {
    log_warn_ctx("services.online.onebot", action, message);
}

#[cfg(feature = "online-tunnel")]
pub(super) fn tunnel_app_data_dir() -> PathBuf {
    crate::utils::path::get_app_data_dir()
}

#[cfg(feature = "online-tunnel")]
pub(super) fn tunnel_translate(key: &str) -> String {
    i18n_service().t(key)
}

#[cfg(feature = "online-tunnel")]
pub(super) fn tunnel_translate_with_args(key: &str, args: &[String]) -> String {
    let values = args
        .iter()
        .enumerate()
        .map(|(index, value)| (index.to_string(), value.clone()))
        .collect::<HashMap<_, _>>();
    i18n_service().t_with_options(key, &values)
}
