//! 对 iroh 公共类型的 newtype 封装。

use std::fmt;
use std::str::FromStr;

/// Relay 服务器地址，封装 [`iroh::RelayUrl`]。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelayUrl(pub(crate) iroh::RelayUrl);

impl fmt::Display for RelayUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for RelayUrl {
    type Err = iroh::RelayUrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<iroh::RelayUrl>().map(Self)
    }
}

impl From<iroh::RelayUrl> for RelayUrl {
    fn from(url: iroh::RelayUrl) -> Self {
        Self(url)
    }
}

/// 节点密钥，封装 [`iroh::SecretKey`]。
#[derive(Debug, Clone)]
pub struct SecretKey(pub(crate) iroh::SecretKey);

impl SecretKey {
    /// 从 32 字节数组创建密钥。
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(iroh::SecretKey::from_bytes(bytes))
    }

    /// 导出 32 字节数组。
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// 获取对应的公钥。
    pub fn public(&self) -> iroh::EndpointId {
        self.0.public()
    }
}

impl From<iroh::SecretKey> for SecretKey {
    fn from(key: iroh::SecretKey) -> Self {
        Self(key)
    }
}
