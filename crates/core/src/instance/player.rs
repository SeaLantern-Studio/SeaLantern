use std::fmt;

/// Minecraft 玩家名称的已验证值对象。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn new(value: impl Into<String>) -> Result<Self, PlayerNameError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(PlayerNameError::Empty);
        }
        if value.len() > 16
            || !value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(PlayerNameError::Invalid { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 玩家管理页面共用的玩家状态快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub name: PlayerName,
    pub uuid: Option<String>,
    pub operator_level: Option<u8>,
    pub banned: bool,
    pub whitelisted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerNameError {
    Empty,
    Invalid { value: String },
}

impl fmt::Display for PlayerNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "player name cannot be empty"),
            Self::Invalid { value } => write!(formatter, "invalid player name: {value}"),
        }
    }
}

impl std::error::Error for PlayerNameError {}

#[cfg(test)]
mod tests {
    use super::{PlayerName, PlayerNameError};

    #[test]
    fn player_name_trims_and_rejects_invalid_characters() {
        assert_eq!(PlayerName::new(" Steve ").unwrap().as_str(), "Steve");
        assert!(matches!(PlayerName::new(""), Err(PlayerNameError::Empty)));
        assert!(matches!(PlayerName::new("bad name"), Err(PlayerNameError::Invalid { .. })));
    }
}
