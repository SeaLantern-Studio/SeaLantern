use regex::Regex;

/// 解析后的版本结构体
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedVersion {
    pub core: [u64; 3],
    pub pre: Option<Vec<PreIdent>>,
}

/// 预发布标识符
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreIdent {
    Numeric(u64),
    AlphaNum(String),
}

impl Ord for PreIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            (Self::Numeric(a), Self::Numeric(b)) => a.cmp(b),
            (Self::Numeric(_), Self::AlphaNum(_)) => Ordering::Less,
            (Self::AlphaNum(_), Self::Numeric(_)) => Ordering::Greater,
            (Self::AlphaNum(a), Self::AlphaNum(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for PreIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParsedVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match self.core.cmp(&other.core) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => {
                for i in 0..std::cmp::max(a.len(), b.len()) {
                    match (a.get(i), b.get(i)) {
                        (Some(x), Some(y)) => match x.cmp(y) {
                            Ordering::Equal => continue,
                            ord => return ord,
                        },
                        (Some(_), None) => return Ordering::Greater,
                        (None, Some(_)) => return Ordering::Less,
                        (None, None) => return Ordering::Equal,
                    }
                }

                Ordering::Equal
            }
        }
    }
}

impl PartialOrd for ParsedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// 比较两个版本，如果 latest 比 current 新则返回 true
pub fn compare_versions(current: &str, latest: &str) -> bool {
    let current_v = parse_version(current);
    let latest_v = parse_version(latest);
    latest_v > current_v
}

/// 解析版本字符串
pub fn parse_version(input: &str) -> ParsedVersion {
    let normalized = input.trim().trim_start_matches(['v', 'V']);
    let no_build = normalized.split('+').next().unwrap_or(normalized);

    let (core_part, pre_part) = no_build
        .split_once('-')
        .map_or((no_build, None), |(core, pre)| (core, Some(pre)));

    let mut core = [0_u64; 3];
    for (idx, piece) in core_part.split('.').take(3).enumerate() {
        core[idx] = piece.trim().parse::<u64>().unwrap_or(0);
    }

    let pre = pre_part.and_then(|p| {
        let idents: Vec<PreIdent> = p
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| match s.parse::<u64>() {
                Ok(n) => PreIdent::Numeric(n),
                Err(_) => PreIdent::AlphaNum(s.to_ascii_lowercase()),
            })
            .collect();

        if idents.is_empty() {
            None
        } else {
            Some(idents)
        }
    });

    ParsedVersion { core, pre }
}

/// 规范化发布标签版本号
pub fn normalize_release_tag_version(tag_name: &str) -> String {
    let trimmed = tag_name.trim();
    if let Some(extracted) = extract_semver_from_text(trimmed) {
        return extracted;
    }

    trimmed.trim_start_matches(['v', 'V']).to_string()
}

/// 从文本中提取语义化版本号
fn extract_semver_from_text(input: &str) -> Option<String> {
    let regex =
        Regex::new(r"(?i)\bv?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)\b").ok()?;

    let mut last_match: Option<String> = None;
    for captures in regex.captures_iter(input) {
        if let Some(matched) = captures.get(1) {
            last_match = Some(matched.as_str().to_string());
        }
    }

    last_match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_handles_prerelease() {
        assert!(compare_versions("1.2.3-beta.1", "1.2.3"));
        assert!(!compare_versions("1.2.3", "1.2.3-beta.1"));
        assert!(compare_versions("1.2.3-beta.1", "1.2.3-beta.2"));
        assert!(!compare_versions("1.2.3-rc.2", "1.2.3-rc.1"));
    }

    #[test]
    fn compare_versions_handles_basic_semver() {
        assert!(compare_versions("1.2.3", "1.2.4"));
        assert!(!compare_versions("1.2.4", "1.2.3"));
        assert!(compare_versions("v1.9.9", "2.0.0"));
        assert!(!compare_versions("2.0.0", "2.0.0"));
    }

    #[test]
    fn parse_version_ignores_build_metadata() {
        assert_eq!(parse_version("1.2.3+abc"), parse_version("1.2.3+def"));
    }

    #[test]
    fn normalize_release_tag_version_handles_prefixed_tag() {
        assert_eq!(normalize_release_tag_version("sea-lantern-v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_plain_version_tag() {
        assert_eq!(normalize_release_tag_version("v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_prerelease_tag() {
        assert_eq!(normalize_release_tag_version("SeaLantern_release-v1.2.3-rc.1"), "1.2.3-rc.1");
    }
}
