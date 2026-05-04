use super::*;

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
