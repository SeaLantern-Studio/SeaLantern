use uuid::Uuid;

/// 生成随机短 ID
pub(super) fn generate_id() -> String {
    format!("srv-{}", Uuid::new_v4().to_string()[..8].to_lowercase())
}

/// 按名称生成短 ID
pub(super) fn generate_id_from_name(name: &str) -> String {
    let sanitized = name
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    if sanitized.is_empty() {
        return generate_id();
    }

    format!("{}-{}", sanitized, Uuid::new_v4().to_string()[..4].to_lowercase())
}

/// 校验自定义短 ID
pub(super) fn validate_custom_id(custom_id: &str) -> Result<(), String> {
    if custom_id.len() < 3 || custom_id.len() > 32 {
        return Err("Server ID must be between 3 and 32 characters".to_string());
    }
    if !custom_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Server ID can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        );
    }
    Ok(())
}

/// 当前 Unix 时间戳
pub(super) fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
