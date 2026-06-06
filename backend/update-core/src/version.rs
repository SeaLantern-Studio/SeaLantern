use regex::Regex;

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParsedVersion {
    core: [u64; 3],
    pre: Option<Vec<PreIdent>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum PreIdent {
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

#[cfg_attr(debug_assertions, allow(dead_code))]
pub fn compare_versions(current: &str, latest: &str) -> bool {
    compare_versions_checked(current, latest).unwrap_or(false)
}

pub fn compare_versions_checked(current: &str, latest: &str) -> Result<bool, String> {
    let current_v = parse_version_checked(current)?;
    let latest_v = parse_version_checked(latest)?;
    Ok(latest_v > current_v)
}

#[cfg(test)]
fn parse_version(input: &str) -> ParsedVersion {
    parse_version_checked(input).unwrap_or(ParsedVersion {
        core: [0, 0, 0],
        pre: None,
    })
}

fn parse_version_checked(input: &str) -> Result<ParsedVersion, String> {
    let normalized = input.trim().trim_start_matches(['v', 'V']);
    if normalized.is_empty() {
        return Err("版本号不能为空".to_string());
    }

    let no_build = normalized.split('+').next().unwrap_or(normalized);

    let (core_part, pre_part) = no_build
        .split_once('-')
        .map_or((no_build, None), |(core, pre)| (core, Some(pre)));

    if core_part.trim().is_empty() {
        return Err(format!("版本号无效 '{}': 缺少核心版本号", input));
    }

    let mut core = [0_u64; 3];
    for (idx, piece) in core_part.split('.').take(3).enumerate() {
        let trimmed = piece.trim();
        if trimmed.is_empty() {
            return Err(format!("版本号无效 '{}': 包含空的核心版本段", input));
        }

        core[idx] = trimmed
            .parse::<u64>()
            .map_err(|e| format!("版本号无效 '{}': 核心版本段 '{}' 解析失败: {}", input, trimmed, e))?;
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

    Ok(ParsedVersion { core, pre })
}

pub fn normalize_release_tag_version(tag_name: &str) -> String {
    let trimmed = tag_name.trim();
    if let Some(extracted) = extract_semver_from_text(trimmed) {
        return extracted;
    }

    trimmed.trim_start_matches(['v', 'V']).to_string()
}

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
#[path = "version_tests.rs"]
mod tests;
