/// mclo.gs 限制单条日志最多 25,000 行，超出会上传失败。
const MAX_LOG_LINE_COUNT: usize = 25_000;
/// mclo.gs 限制单条日志最多约 10 MiB（UTF-8 字节长度），超出会上传失败。
const MAX_LOG_SIZE_BYTES: usize = 10 * 1024 * 1024;
/// tracing 目标字段，便于按模块过滤日志。
const TRACING_TARGET: &str = "sealantern.extra.mclogs";

/// 超出行数限制时仅保留最后 `MAX_LOG_LINE_COUNT` 行。
///
/// 先统计总行数，再用 `skip` 顺序取末尾 N 行，
/// 避免双向反转与多余的中间分配。
fn truncate_to_last_lines(text: &str) -> String {
    let total_lines = text.lines().count();
    if total_lines > MAX_LOG_LINE_COUNT {
        let dropped_lines = total_lines - MAX_LOG_LINE_COUNT;
        tracing::debug!(
            target: TRACING_TARGET,
            dropped_lines,
            kept_lines = MAX_LOG_LINE_COUNT,
            "log exceeds mclo.gs line limit, keeping only the last lines"
        );
        text.lines()
            .skip(total_lines - MAX_LOG_LINE_COUNT)
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        text.to_string()
    }
}

/// 清理日志并应用 mclo.gs 的行数及 UTF-8 字节大小限制。
pub(super) fn prepare_payload(content: &str) -> Result<String, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        tracing::debug!(target: TRACING_TARGET, "share_logs rejected: empty content");
        return Err("日志内容为空，无法分享".to_string());
    }

    // 行数截断（保留末尾 N 行）。
    let payload_content = truncate_to_last_lines(trimmed);

    // 大小限制：mclo.gs 单条上限约 10 MiB，超出会被拒绝，
    // 先本地拦截以给出明确错误，而不是等到服务端返回失败。
    if payload_content.len() > MAX_LOG_SIZE_BYTES {
        tracing::warn!(
            target: TRACING_TARGET,
            size_bytes = payload_content.len(),
            max_bytes = MAX_LOG_SIZE_BYTES,
            "share_logs rejected: content exceeds mclo.gs size limit"
        );
        return Err(format!(
            "日志大小 {} 字节已超过 mclo.gs 上限 {} 字节，无法分享",
            payload_content.len(),
            MAX_LOG_SIZE_BYTES
        ));
    }

    Ok(payload_content)
}

#[cfg(test)]
mod tests {
    use super::{prepare_payload, truncate_to_last_lines, MAX_LOG_LINE_COUNT};

    #[test]
    fn prepare_payload_trims_surrounding_whitespace() {
        assert_eq!(prepare_payload("  第一行\n第二行  "), Ok("第一行\n第二行".to_string()));
    }

    #[test]
    fn truncate_to_last_lines_keeps_the_newest_lines() {
        let content = (0..MAX_LOG_LINE_COUNT + 2)
            .map(|line| format!("line-{line}"))
            .collect::<Vec<_>>()
            .join("\n");

        let truncated = truncate_to_last_lines(&content);
        let expected_last_line = format!("line-{}", MAX_LOG_LINE_COUNT + 1);

        assert_eq!(truncated.lines().count(), MAX_LOG_LINE_COUNT);
        assert_eq!(truncated.lines().next(), Some("line-2"));
        assert_eq!(truncated.lines().last(), Some(expected_last_line.as_str()));
    }
}
