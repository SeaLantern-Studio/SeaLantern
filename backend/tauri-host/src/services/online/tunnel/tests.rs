use super::config::apply_relay_preference;
use super::config::persist_profile_update;
use super::runtime::map_connections_checked;
use super::state::{derive_ticket_for_state_checked, TunnelRuntimeState};
use super::TunnelConnection;
use crate::test_support::{lock_env, EnvGuard};
use sculk::persist::Profile;

#[test]
fn apply_relay_preference_rejects_invalid_input_without_mutating_profile() {
    let mut profile = Profile::default();
    profile.relay.custom = true;
    profile.relay.url = Some("https://relay.example.com".to_string());
    let before = profile.clone();

    let result = apply_relay_preference(&mut profile, Some("not a relay url".to_string()));

    assert!(result.is_err());
    assert_eq!(profile.relay.custom, before.relay.custom);
    assert_eq!(profile.relay.url, before.relay.url);
}

#[test]
fn apply_relay_preference_clears_custom_relay_when_input_empty() {
    let mut profile = Profile::default();
    profile.relay.custom = true;
    profile.relay.url = Some("https://relay.example.com".to_string());

    let result = apply_relay_preference(&mut profile, None);

    assert!(result.is_ok());
    assert!(!profile.relay.custom);
    assert!(profile.relay.url.is_none());
}

#[test]
fn persist_profile_update_surfaces_save_failures_without_mutating_state() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let blocked_root = temp_dir.path().join("blocked-root");
    std::fs::write(&blocked_root, b"not a directory")
        .expect("file-backed app data root should exist");
    let blocked_path = blocked_root.join("nested");

    let _env_lock = lock_env();
    let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

    let mut state = TunnelRuntimeState {
        profile: Profile::default(),
        ..TunnelRuntimeState::default()
    };
    let before = state.profile.clone();

    let error = persist_profile_update(&mut state, |profile| {
        profile.host.port = 25565;
        Ok(())
    })
    .expect_err("profile save failure should not be silently downgraded");

    assert!(
        error.contains("保存 sculk Profile 失败") || error.contains("Failed to save sculk profile"),
        "unexpected error: {}",
        error
    );
    assert_eq!(state.profile.host.port, before.host.port);
    assert_eq!(state.profile.join.port, before.join.port);
    assert_eq!(state.profile.join.last_ticket, before.join.last_ticket);
    assert_eq!(state.profile.relay.custom, before.relay.custom);
    assert_eq!(state.profile.relay.url, before.relay.url);
}

#[test]
fn map_connections_checked_surfaces_query_failures() {
    let error = map_connections_checked::<u8>(Err("boom"), |_| TunnelConnection {
        remote_id: String::new(),
        is_relay: false,
        rtt_ms: 0,
        tx_bytes: 0,
        rx_bytes: 0,
        alive: false,
        elapsed_secs: 0,
    })
    .expect_err("connection query failure should not be downgraded to empty list");

    assert!(
        error.contains("failed to query tunnel connections") || error.contains("获取隧道连接失败"),
        "unexpected error: {error}"
    );
}

#[test]
fn map_connections_checked_keeps_successful_items() {
    let connections =
        map_connections_checked(Ok::<Vec<u8>, &str>(vec![1_u8, 2_u8]), |value| TunnelConnection {
            remote_id: format!("peer-{value}"),
            is_relay: value % 2 == 0,
            rtt_ms: u64::from(value),
            tx_bytes: 10,
            rx_bytes: 20,
            alive: true,
            elapsed_secs: 30,
        })
        .expect("successful connection query should be preserved");

    assert_eq!(connections.len(), 2);
    assert_eq!(connections[0].remote_id, "peer-1");
    assert_eq!(connections[1].remote_id, "peer-2");
    assert!(connections[1].is_relay);
}

#[test]
fn derive_ticket_for_state_checked_surfaces_invalid_custom_relay() {
    let mut state = TunnelRuntimeState {
        profile: Profile::default(),
        ..TunnelRuntimeState::default()
    };
    state.profile.relay.custom = true;
    state.profile.relay.url = Some("not a relay url".to_string());
    state.secret_key = Some(sculk::types::SecretKey::from_bytes(&[7_u8; 32]));

    let error = derive_ticket_for_state_checked(&state)
        .expect_err("invalid custom relay should not silently downgrade ticket derivation");

    assert!(
        error.contains("failed to derive tunnel ticket relay"),
        "unexpected error: {error}"
    );
}
