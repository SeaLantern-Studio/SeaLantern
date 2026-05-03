use super::i18n::tunnel_t1;
use super::state::{derive_ticket_for_state, tunnel_key_path, tunnel_profile_path, TunnelRuntimeState};
use sculk::persist::{generate_new_key, Profile};
use sculk::types::{RelayUrl, SecretKey};

pub(super) fn load_existing_secret_key(path: &std::path::Path) -> Result<Option<SecretKey>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(tunnel_t1("tunnel.err.read_key_failed", e.to_string()));
        }
    };
    if bytes.len() != 32 {
        return Err(tunnel_t1("tunnel.err.key_length_invalid", format!("{}", bytes.len())));
    }
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|v: Vec<u8>| tunnel_t1("tunnel.err.key_length_invalid", format!("{}", v.len())))?;
    Ok(Some(SecretKey::from_bytes(&arr)))
}

pub(super) fn load_runtime_state() -> TunnelRuntimeState {
    let mut logs = Vec::new();
    let profile_path = tunnel_profile_path();
    let profile = match Profile::load_from(&profile_path) {
        Ok(profile) => profile,
        Err(e) => {
            logs.push(tunnel_t1("tunnel.log.load_profile_failed", e.to_string()));
            Profile::default()
        }
    };
    let key_path = tunnel_key_path();
    let secret_key = match load_existing_secret_key(&key_path) {
        Ok(key) => key,
        Err(e) => {
            logs.push(tunnel_t1("tunnel.log.load_secret_key_failed", e.to_string()));
            None
        }
    };

    let mut state = TunnelRuntimeState {
        mode: None,
        ticket: None,
        logs,
        profile,
        secret_key,
    };
    state.ticket = derive_ticket_for_state(&state);
    state
}

pub(super) fn save_profile_in_state(state: &mut TunnelRuntimeState) {
    if let Err(e) = state.profile.save_to(&tunnel_profile_path()) {
        state
            .logs
            .push(tunnel_t1("tunnel.log.save_profile_failed", e.to_string()));
    }
}

pub(super) fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub(super) fn parse_relay_url(value: Option<String>) -> Result<Option<RelayUrl>, String> {
    let Some(raw) = normalize_optional_string(value) else {
        return Ok(None);
    };
    raw.parse::<RelayUrl>()
        .map(Some)
        .map_err(|e| tunnel_t1("tunnel.err.invalid_relay_url", e.to_string()))
}

pub(super) fn apply_relay_preference(
    profile: &mut Profile,
    relay_input: Option<String>,
) -> Result<Option<RelayUrl>, String> {
    let normalized = normalize_optional_string(relay_input);
    let parsed = parse_relay_url(normalized.clone())?;

    if let Some(url) = normalized {
        profile.relay.custom = true;
        profile.relay.url = Some(url);
        Ok(parsed)
    } else {
        profile.relay.custom = false;
        profile.relay.url = None;
        Ok(None)
    }
}

pub(super) fn ensure_secret_key(state: &mut TunnelRuntimeState) -> Result<SecretKey, String> {
    if state.secret_key.is_none() {
        let key = generate_new_key(&tunnel_key_path())
            .map_err(|e| tunnel_t1("tunnel.err.generate_key_failed", e.to_string()))?;
        state.secret_key = Some(key);
        state.ticket = derive_ticket_for_state(state);
    }

    state
        .secret_key
        .clone()
        .ok_or_else(|| tunnel_t1("tunnel.err.generate_key_failed", "missing secret key"))
}
