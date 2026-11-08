//! 系统字体族枚举。
//!
//! 经 `font-kit` 读取系统字体目录，返回去重后的字体族名，供设置页面的字体选择使用。

use super::error::PlatformError;

/// 采集系统可用字体族名（去重、按不分大小写升序排序）。
///
/// 枚举失败时返回 [`PlatformError::FontEnumerationFailed`]。
pub fn collect_system_fonts() -> Result<Vec<String>, PlatformError> {
    use std::collections::HashSet;

    let source = font_kit::source::SystemSource::new();
    let families = source
        .all_families()
        .map_err(|error| PlatformError::FontEnumerationFailed { message: error.to_string() })?;

    let unique: HashSet<String> = families.into_iter().collect();
    let mut fonts: Vec<String> = unique.into_iter().collect();
    fonts.sort_by_key(|name| name.to_lowercase());
    Ok(fonts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_font_families_sorted_and_unique() {
        let fonts = collect_system_fonts().expect("enumerating system fonts must succeed");

        // 结果应按不分大小写升序排序（复用同一排序规则比对）。
        let mut sorted = fonts.clone();
        sorted.sort_by_key(|name| name.to_lowercase());
        assert_eq!(fonts, sorted, "font families must be sorted case-insensitively");

        // 排序后相邻去重应不改变长度，即结果已去重。
        let mut unique = fonts.clone();
        unique.dedup();
        assert_eq!(fonts.len(), unique.len(), "font families must be unique");
    }
}
