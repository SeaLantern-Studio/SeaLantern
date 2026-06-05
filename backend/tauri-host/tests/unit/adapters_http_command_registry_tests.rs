use super::*;

#[test]
fn command_registry_includes_tunnel_commands() {
    let registry = CommandRegistry::new();
    let commands = registry.list_commands();

    assert!(commands.contains(&"tunnel_host".to_string()));
    assert!(commands.contains(&"tunnel_join".to_string()));
    assert!(commands.contains(&"tunnel_stop".to_string()));
    assert!(commands.contains(&"tunnel_status".to_string()));
    assert!(commands.contains(&"tunnel_copy_ticket".to_string()));
    assert!(commands.contains(&"tunnel_regenerate_ticket".to_string()));
    assert!(commands.contains(&"tunnel_generate_ticket".to_string()));
}

#[test]
fn command_registry_includes_preview_from_source_command() {
    let registry = CommandRegistry::new();
    let commands = registry.list_commands();

    assert!(commands.contains(&"preview_server_properties_write_from_source".to_string()));
}
