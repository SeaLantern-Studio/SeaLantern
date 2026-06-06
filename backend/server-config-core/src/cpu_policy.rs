use crate::types::{CpuPolicyConfig, CpuPolicyMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCpuPolicy {
    pub cpu_indices: Vec<usize>,
    pub cpuset_display: String,
    pub active_processor_count: u16,
}

pub fn resolve_cpu_policy(policy: &CpuPolicyConfig) -> Result<Option<ResolvedCpuPolicy>, String> {
    resolve_cpu_policy_with_limit(policy, None)
}

pub fn resolve_local_cpu_policy(
    policy: &CpuPolicyConfig,
    logical_cpu_count: usize,
) -> Result<Option<ResolvedCpuPolicy>, String> {
    resolve_cpu_policy_with_limit(policy, Some(logical_cpu_count))
}

fn resolve_cpu_policy_with_limit(
    policy: &CpuPolicyConfig,
    logical_cpu_count: Option<usize>,
) -> Result<Option<ResolvedCpuPolicy>, String> {
    match policy.mode {
        CpuPolicyMode::Off => Ok(None),
        CpuPolicyMode::Count => {
            let count = policy
                .count
                .ok_or_else(|| "CPU 限制 count 模式缺少 count".to_string())?;
            if count == 0 {
                return Err("CPU 限制 count 模式必须大于 0".to_string());
            }

            if let Some(logical_cpu_count) = logical_cpu_count {
                if usize::from(count) > logical_cpu_count {
                    return Err(format!(
                        "CPU 限制 count={} 超过当前主机逻辑 CPU 数 {}",
                        count, logical_cpu_count
                    ));
                }
            }

            let cpu_indices = (0..usize::from(count)).collect::<Vec<_>>();
            Ok(Some(ResolvedCpuPolicy {
                cpuset_display: format_cpu_range(&cpu_indices),
                cpu_indices,
                active_processor_count: count,
            }))
        }
        CpuPolicyMode::Explicit => {
            let raw = policy
                .explicit_set
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "CPU 限制 explicit 模式缺少 explicit_set".to_string())?;

            let cpu_indices = parse_cpu_set(raw)?;
            if let Some(logical_cpu_count) = logical_cpu_count {
                if cpu_indices.iter().any(|index| *index >= logical_cpu_count) {
                    return Err(format!(
                        "CPU 核心集合超出当前主机逻辑 CPU 数 {}: {}",
                        logical_cpu_count, raw
                    ));
                }
            }

            if cpu_indices.is_empty() {
                return Err("CPU 核心集合解析后为空".to_string());
            }

            let active_processor_count =
                u16::try_from(cpu_indices.len()).map_err(|_| "CPU 核心集合数量过大".to_string())?;
            Ok(Some(ResolvedCpuPolicy {
                cpuset_display: format_cpu_range(&cpu_indices),
                cpu_indices,
                active_processor_count,
            }))
        }
    }
}

pub fn resolve_active_processor_count(
    policy: &CpuPolicyConfig,
    logical_cpu_count: usize,
) -> Result<Option<u16>, String> {
    if policy.mode == CpuPolicyMode::Off || !policy.sync_active_processor_count {
        return Ok(None);
    }

    Ok(resolve_local_cpu_policy(policy, logical_cpu_count)?
        .map(|resolved| resolved.active_processor_count))
}

pub fn resolve_unbounded_active_processor_count(
    policy: &CpuPolicyConfig,
) -> Result<Option<u16>, String> {
    if policy.mode == CpuPolicyMode::Off || !policy.sync_active_processor_count {
        return Ok(None);
    }

    Ok(resolve_cpu_policy(policy)?.map(|resolved| resolved.active_processor_count))
}

pub fn parse_cpu_set(raw: &str) -> Result<Vec<usize>, String> {
    let mut values = Vec::new();
    for chunk in raw.split(',') {
        let token = chunk.trim();
        if token.is_empty() {
            return Err(format!("CPU 核心集合格式无效: {}", raw));
        }

        if let Some((start_raw, end_raw)) = token.split_once('-') {
            let start = parse_cpu_index(start_raw, raw)?;
            let end = parse_cpu_index(end_raw, raw)?;
            if end < start {
                return Err(format!("CPU 核心区间无效: {}", raw));
            }
            values.extend(start..=end);
        } else {
            values.push(parse_cpu_index(token, raw)?);
        }
    }

    values.sort_unstable();
    values.dedup();
    Ok(values)
}

pub fn format_cpu_range(indices: &[usize]) -> String {
    if indices.is_empty() {
        return String::new();
    }

    let mut ranges = Vec::new();
    let mut start = indices[0];
    let mut prev = indices[0];

    for &value in &indices[1..] {
        if value == prev + 1 {
            prev = value;
            continue;
        }

        ranges.push(format_cpu_range_segment(start, prev));
        start = value;
        prev = value;
    }

    ranges.push(format_cpu_range_segment(start, prev));
    ranges.join(",")
}

fn parse_cpu_index(raw: &str, whole: &str) -> Result<usize, String> {
    raw.trim()
        .parse::<usize>()
        .map_err(|_| format!("CPU 核心集合格式无效: {}", whole))
}

fn format_cpu_range_segment(start: usize, end: usize) -> String {
    if start == end {
        start.to_string()
    } else {
        format!("{}-{}", start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_cpu_range, parse_cpu_set, resolve_active_processor_count, resolve_cpu_policy,
        resolve_local_cpu_policy, resolve_unbounded_active_processor_count,
    };
    use crate::types::{CpuPolicyConfig, CpuPolicyMode};

    #[test]
    fn parse_cpu_set_supports_ranges_and_lists() {
        assert_eq!(parse_cpu_set("0-3,6,7").unwrap(), vec![0, 1, 2, 3, 6, 7]);
    }

    #[test]
    fn format_cpu_range_compacts_consecutive_values() {
        assert_eq!(format_cpu_range(&[0, 1, 2, 4, 6, 7]), "0-2,4,6-7");
    }

    #[test]
    fn resolve_local_cpu_policy_supports_count_mode() {
        let resolved = resolve_local_cpu_policy(
            &CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            8,
        )
        .unwrap()
        .unwrap();

        assert_eq!(resolved.cpu_indices, vec![0, 1]);
        assert_eq!(resolved.cpuset_display, "0-1");
        assert_eq!(resolved.active_processor_count, 2);
    }

    #[test]
    fn resolve_local_cpu_policy_supports_explicit_mode() {
        let resolved = resolve_local_cpu_policy(
            &CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("1,3,5".to_string()),
                sync_active_processor_count: true,
            },
            8,
        )
        .unwrap()
        .unwrap();

        assert_eq!(resolved.cpu_indices, vec![1, 3, 5]);
        assert_eq!(resolved.cpuset_display, "1,3,5");
        assert_eq!(resolved.active_processor_count, 3);
    }

    #[test]
    fn resolve_cpu_policy_supports_unbounded_count_mode() {
        let resolved = resolve_cpu_policy(
            &CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(32),
                explicit_set: None,
                sync_active_processor_count: true,
            },
        )
        .unwrap()
        .unwrap();

        assert_eq!(resolved.cpuset_display, "0-31");
        assert_eq!(resolved.active_processor_count, 32);
    }

    #[test]
    fn resolve_active_processor_count_respects_sync_flag() {
        assert_eq!(
            resolve_active_processor_count(
                &CpuPolicyConfig {
                    mode: CpuPolicyMode::Count,
                    count: Some(2),
                    explicit_set: None,
                    sync_active_processor_count: false,
                },
                8,
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn resolve_unbounded_active_processor_count_uses_explicit_cpu_set_size() {
        assert_eq!(
            resolve_unbounded_active_processor_count(&CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("1,3,5".to_string()),
                sync_active_processor_count: true,
            })
            .unwrap(),
            Some(3)
        );
    }

    #[test]
    fn resolve_local_cpu_policy_rejects_out_of_range_indices() {
        let err = resolve_local_cpu_policy(
            &CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("0,8".to_string()),
                sync_active_processor_count: true,
            },
            8,
        )
        .expect_err("out of range explicit set should fail");

        assert!(err.contains("超出当前主机逻辑 CPU 数"));
    }
}
