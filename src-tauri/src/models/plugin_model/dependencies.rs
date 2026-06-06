use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// 简化版语义化版本
///
/// 用来处理插件依赖里的版本比较和范围判断
#[derive(Debug, Clone, Default)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
}

impl SemVer {
    /// 解析版本字符串
    ///
    /// # Parameters
    ///
    /// - `version`: 版本字符串，允许带 `v` 前缀
    ///
    /// # Returns
    ///
    /// 解析成功时返回版本结构，失败时返回 `None`
    pub fn parse(version: &str) -> Option<Self> {
        let clean = version.trim().trim_start_matches('v');
        if clean.is_empty() {
            return None;
        }
        let parts: Vec<&str> = clean.split('.').collect();

        let major = parts.first().and_then(|s| s.parse().ok())?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        let (patch, prerelease) = if let Some(patch_str) = parts.get(2) {
            if let Some(idx) = patch_str.find('-') {
                let (p, pre) = patch_str.split_at(idx);
                (p.parse().unwrap_or(0), Some(pre[1..].to_string()))
            } else {
                (patch_str.parse().unwrap_or(0), None)
            }
        } else {
            (0, None)
        };
        Some(Self { major, minor, patch, prerelease })
    }

    /// 判断当前版本是否满足依赖要求
    ///
    /// # Parameters
    ///
    /// - `requirement`: 依赖要求，例如 `>=1.2.0`、`^2.0.0`
    ///
    /// # Returns
    ///
    /// 满足时返回 `true`
    pub fn satisfies(&self, requirement: &str) -> bool {
        let req = requirement.trim();

        if let Some(stripped) = req.strip_prefix(">=") {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) != Ordering::Less;
            }
        } else if let Some(stripped) = req.strip_prefix('>') {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) == Ordering::Greater;
            }
        } else if let Some(stripped) = req.strip_prefix("<=") {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) != Ordering::Greater;
            }
        } else if let Some(stripped) = req.strip_prefix('<') {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) == Ordering::Less;
            }
        } else if let Some(stripped) = req.strip_prefix('=') {
            if let Some(target) = SemVer::parse(stripped) {
                return self.compare(&target) == Ordering::Equal;
            }
        } else if let Some(stripped) = req.strip_prefix('^') {
            if let Some(target) = SemVer::parse(stripped) {
                if self.compare(&target) == Ordering::Less {
                    return false;
                }
                if target.major == 0 {
                    if target.minor == 0 {
                        return self.major == 0 && self.minor == 0 && self.patch == target.patch;
                    }
                    return self.major == 0 && self.minor == target.minor;
                }
                return self.major == target.major;
            }
        } else if let Some(stripped) = req.strip_prefix('~') {
            if let Some(target) = SemVer::parse(stripped) {
                if self.compare(&target) == Ordering::Less {
                    return false;
                }
                return self.major == target.major && self.minor == target.minor;
            }
        } else if let Some(target) = SemVer::parse(req) {
            return self.compare(&target) == Ordering::Equal;
        }

        false
    }

    fn compare(&self, other: &SemVer) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// 插件依赖声明
///
/// 支持只写插件 ID，也支持带版本要求的写法
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginDependency {
    Simple(String),
    WithVersion {
        id: String,
        #[serde(default)]
        version: Option<String>,
    },
}

impl PluginDependency {
    /// 读取依赖的插件 ID
    ///
    /// # Returns
    ///
    /// 返回依赖目标的插件 ID
    pub fn id(&self) -> &str {
        match self {
            Self::Simple(id) => id,
            Self::WithVersion { id, .. } => id,
        }
    }

    /// 读取版本要求
    ///
    /// # Returns
    ///
    /// 如果依赖里声明了版本要求，则返回它
    pub fn version_requirement(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::WithVersion { version, .. } => version.as_deref(),
        }
    }

    /// 判断一个插件版本是否满足当前依赖
    ///
    /// # Parameters
    ///
    /// - `version`: 待检查的插件版本
    ///
    /// # Returns
    ///
    /// 满足依赖要求时返回 `true`
    pub fn is_satisfied_by(&self, version: &str) -> bool {
        match self.version_requirement() {
            Some(req) => SemVer::parse(version).is_some_and(|ver| ver.satisfies(req)),
            None => true,
        }
    }
}
