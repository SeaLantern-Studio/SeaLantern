use crate::models::settings::{PartialSettings, UI_SHELL_CLASSIC, UI_SHELL_NEXT};
use crate::services::global;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};

const DESKTOP_PRIMARY_SHELL: &str = UI_SHELL_NEXT;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UiShellInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub builtin: bool,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UiShellStatus {
    pub configured_shell: String,
    pub effective_shell: String,
    pub pending_restart: bool,
    pub available_shells: Vec<UiShellInfo>,
}

#[derive(Debug, Default)]
struct UiShellRuntimeState {
    reported_shell: Mutex<Option<String>>,
}

fn runtime_state() -> &'static UiShellRuntimeState {
    static INSTANCE: OnceLock<UiShellRuntimeState> = OnceLock::new();
    INSTANCE.get_or_init(UiShellRuntimeState::default)
}

fn normalize_shell_id(shell_id: &str) -> Option<&'static str> {
    match shell_id.trim().to_ascii_lowercase().as_str() {
        UI_SHELL_CLASSIC => Some(UI_SHELL_CLASSIC),
        UI_SHELL_NEXT => Some(UI_SHELL_NEXT),
        _ => None,
    }
}

fn ui_shell_label(shell_id: &str) -> &'static str {
    match shell_id {
        UI_SHELL_NEXT => "Next",
        _ => "Classic",
    }
}

fn ui_shell_description(shell_id: &str) -> &'static str {
    match shell_id {
        UI_SHELL_NEXT => "Builtin next renderer shell.",
        _ => "Builtin classic fallback shell.",
    }
}

fn available_shell_ids() -> [&'static str; 2] {
    [UI_SHELL_CLASSIC, UI_SHELL_NEXT]
}

fn runtime_reported_shell() -> Option<String> {
    runtime_state()
        .reported_shell
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn compute_effective_shell(configured_shell: &str) -> String {
    if let Some(reported) = runtime_reported_shell() {
        if let Some(valid) = normalize_shell_id(&reported) {
            return valid.to_string();
        }
    }

    let _ = configured_shell;
    DESKTOP_PRIMARY_SHELL.to_string()
}

fn build_available_shells() -> Vec<UiShellInfo> {
    available_shell_ids()
        .into_iter()
        .map(|shell_id| UiShellInfo {
            id: shell_id.to_string(),
            name: ui_shell_label(shell_id).to_string(),
            description: ui_shell_description(shell_id).to_string(),
            source: "builtin".to_string(),
            builtin: true,
            available: shell_id == DESKTOP_PRIMARY_SHELL,
        })
        .collect()
}

pub fn get_status() -> UiShellStatus {
    let configured_shell = global::settings_manager().get().ui_shell;
    let effective_shell = compute_effective_shell(&configured_shell);
    let pending_restart = configured_shell != effective_shell;
    let available_shells = build_available_shells();

    UiShellStatus {
        configured_shell,
        effective_shell,
        pending_restart,
        available_shells,
    }
}

pub fn set_shell(shell_id: &str) -> Result<UiShellStatus, String> {
    let shell_id = normalize_shell_id(shell_id)
        .ok_or_else(|| format!("Unsupported ui shell: {}", shell_id.trim()))?;

    if shell_id != DESKTOP_PRIMARY_SHELL {
        return Err(format!(
            "UI shell is not available as a normal desktop startup target: {}",
            shell_id
        ));
    }

    let status = get_status();
    let is_available = status
        .available_shells
        .iter()
        .any(|shell| shell.id == shell_id && shell.available);

    if !is_available {
        return Err(format!("UI shell is not available in current runtime: {}", shell_id));
    }

    let result = global::settings_manager().update_partial(PartialSettings {
        ui_shell: Some(shell_id.to_string()),
        ..Default::default()
    })?;

    let effective_shell = compute_effective_shell(&result.settings.ui_shell);

    Ok(UiShellStatus {
        configured_shell: result.settings.ui_shell.clone(),
        pending_restart: result.settings.ui_shell != effective_shell,
        effective_shell,
        available_shells: build_available_shells(),
    })
}

pub fn report_runtime(shell_id: &str) -> Result<UiShellStatus, String> {
    let shell_id = normalize_shell_id(shell_id)
        .ok_or_else(|| format!("Unsupported runtime ui shell: {}", shell_id.trim()))?;

    *runtime_state()
        .reported_shell
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(shell_id.to_string());

    Ok(get_status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_shell_id_rejects_unknown_values() {
        assert_eq!(normalize_shell_id("classic"), Some(UI_SHELL_CLASSIC));
        assert_eq!(normalize_shell_id("NEXT"), Some(UI_SHELL_NEXT));
        assert_eq!(normalize_shell_id("unknown"), None);
    }

    #[test]
    fn compute_effective_shell_defaults_to_next_without_runtime_report() {
        *runtime_state()
            .reported_shell
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = None;

        assert_eq!(compute_effective_shell(UI_SHELL_NEXT), UI_SHELL_NEXT);
        assert_eq!(compute_effective_shell(UI_SHELL_CLASSIC), UI_SHELL_NEXT);
    }

    #[test]
    fn build_available_shells_only_exposes_next_as_available() {
        let shells = build_available_shells();
        let next = shells.iter().find(|shell| shell.id == UI_SHELL_NEXT).unwrap();
        let classic = shells
            .iter()
            .find(|shell| shell.id == UI_SHELL_CLASSIC)
            .unwrap();

        assert!(next.available);
        assert!(!classic.available);
    }

    #[test]
    fn report_runtime_keeps_real_reported_shell_even_in_safe_mode() {
        *runtime_state()
            .reported_shell
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = None;

        let status = report_runtime(UI_SHELL_NEXT).expect("report runtime should succeed");

        assert_eq!(status.effective_shell, UI_SHELL_NEXT);
        assert_eq!(runtime_reported_shell().as_deref(), Some(UI_SHELL_NEXT));
    }

    #[test]
    fn set_shell_rejects_classic_as_normal_target() {
        let error = set_shell(UI_SHELL_CLASSIC).expect_err("classic should be rejected");
        assert!(error.contains("normal desktop startup target"));
    }
}
