//! 玩家坐标查询：命令构造与回显解析。
//!
//! 一条命令就能拿到某个维度全部在线玩家的坐标，不需要按玩家逐个查：
//!
//! ```text
//! execute as @a at @s if dimension minecraft:overworld run data get entity @s Pos
//! execute as @a at @s if dimension minecraft:overworld run data get entity @s Rotation
//! ```
//!
//! `at @s` 不能省。`as` 只换执行者，执行上下文的位置与维度仍是命令源自己的：
//! 走 RCON 时命令源是控制台，落在主世界出生点，于是 `if dimension overworld`
//! 恒为真、`if dimension the_nether` 恒为假 —— 前者会把下界玩家一并返回、
//! 画到主世界地图上，后者永远返回空。`at` 才会把上下文的位置、朝向与维度
//! 一起换成目标的。
//!
//! 回显形如：
//!
//! ```text
//! Steve has the following entity data: [123.45d, 64.0d, -678.9d]
//! Steve has the following entity data: [90.0f, 0.0f]
//! ```
//!
//! 这里只做纯字符串处理，不碰网络，方便单元测试。

use sealantern_contract::Dimension;

/// 构造「某维度所有玩家坐标」的命令。
///
/// `at @s` 必须在 `as` 之后：判断维度用的是执行上下文，而只有 `at` 会替换上下文。
pub fn build_positions_command(dimension: Dimension) -> String {
    format!(
        "execute as @a at @s if dimension {} run data get entity @s Pos",
        dimension.vanilla_id()
    )
}

/// 构造「某维度所有玩家朝向」的命令。
///
/// 同 [`build_positions_command`]，`at @s` 负责把上下文维度换到玩家身上。
pub fn build_rotation_command(dimension: Dimension) -> String {
    format!(
        "execute as @a at @s if dimension {} run data get entity @s Rotation",
        dimension.vanilla_id()
    )
}

/// 回显里名字与数据之间的固定分隔。
const SEPARATOR: &str = " has the following entity data: ";

/// 从 `execute ... run data get entity @s Pos` 的回显里解析玩家坐标。
///
/// 返回按玩家名升序、(名字, [x, y, z]) 的列表；解析不出的行直接跳过。
pub fn parse_positions(lines: &[String]) -> Vec<(String, [f64; 3])> {
    let mut parsed: Vec<(String, [f64; 3])> = lines
        .iter()
        .filter_map(|line| {
            let (name, value) = split_line(line)?;
            let numbers = parse_number_list(value)?;
            if numbers.len() != 3 {
                return None;
            }
            Some((name.to_string(), [numbers[0], numbers[1], numbers[2]]))
        })
        .collect();
    parsed.sort_by(|a, b| a.0.cmp(&b.0));
    parsed
}

/// 从 `execute ... run data get entity @s Rotation` 的回显里解析玩家朝向。
///
/// 只取第一个分量（yaw），俯视地图用不到俯仰角。
pub fn parse_rotations(lines: &[String]) -> Vec<(String, f32)> {
    let mut parsed: Vec<(String, f32)> = lines
        .iter()
        .filter_map(|line| {
            let (name, value) = split_line(line)?;
            let numbers = parse_number_list(value)?;
            Some((name.to_string(), numbers.first().copied()? as f32))
        })
        .collect();
    parsed.sort_by(|a, b| a.0.cmp(&b.0));
    parsed
}

/// 拆出玩家名与冒号后的数据部分。
fn split_line(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once(SEPARATOR)?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some((name, value.trim()))
}

/// 解析 `[1.0d, 2.0d, 3.0d]` 这类数组，去掉 Minecraft 的类型后缀。
fn parse_number_list(value: &str) -> Option<Vec<f64>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    let mut numbers = Vec::new();
    for item in inner.split(',') {
        let item = item.trim().trim_end_matches(['d', 'f', 'b', 's', 'l']);
        numbers.push(item.parse::<f64>().ok()?);
    }
    (!numbers.is_empty()).then_some(numbers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|line| line.to_string()).collect()
    }

    #[test]
    fn builds_command_with_dimension_id() {
        assert_eq!(
            build_positions_command(Dimension::Nether),
            "execute as @a at @s if dimension minecraft:the_nether run data get entity @s Pos"
        );
        assert_eq!(
            build_rotation_command(Dimension::End),
            "execute as @a at @s if dimension minecraft:the_end run data get entity @s Rotation"
        );
    }

    /// 回归测试：`at @s` 必须在 `if dimension` 之前，否则维度条件检查的是
    /// RCON 命令源（主世界出生点）而不是玩家本人。
    #[test]
    fn dimension_guard_is_preceded_by_at() {
        for command in [
            build_positions_command(Dimension::Overworld),
            build_rotation_command(Dimension::Overworld),
        ] {
            let at = command
                .find("at @s")
                .unwrap_or_else(|| panic!("缺少 at @s: {command}"));
            let guard = command
                .find("if dimension")
                .unwrap_or_else(|| panic!("缺少 if dimension: {command}"));
            assert!(at < guard, "`at @s` 必须出现在维度判断之前: {command}");
        }
    }

    #[test]
    fn parses_positions_from_vanilla_output() {
        let parsed = parse_positions(&lines(&[
            "Steve has the following entity data: [123.45d, 64.0d, -678.9d]",
            "Alex has the following entity data: [-0.5d, 70.0d, 12.0d]",
        ]));
        assert_eq!(parsed.len(), 2);
        // 按名字升序：Alex 在前
        assert_eq!(parsed[0], ("Alex".to_string(), [-0.5, 70.0, 12.0]));
        assert_eq!(parsed[1], ("Steve".to_string(), [123.45, 64.0, -678.9]));
    }

    #[test]
    fn parses_rotation_as_yaw() {
        let parsed = parse_rotations(&lines(&[
            "Steve has the following entity data: [90.0f, 0.0f]",
            "Alex has the following entity data: [-135.5f, 12.0f]",
        ]));
        assert_eq!(parsed, vec![("Alex".to_string(), -135.5f32), ("Steve".to_string(), 90.0f32),]);
    }

    #[test]
    fn ignores_unrelated_and_empty_lines() {
        let parsed = parse_positions(&lines(&[
            "",
            "There are 0 of a max of 20 players online",
            "Steve has the following entity data: [1.0d, 2.0d, 3.0d]",
        ]));
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, "Steve");
    }

    #[test]
    fn rejects_malformed_payloads() {
        // 长度不对、没有方括号、数字非法，都应被跳过而不是 panic
        assert!(
            parse_positions(&lines(&["Steve has the following entity data: [1.0d]"])).is_empty()
        );
        assert!(parse_positions(&lines(&["Steve has the following entity data: 1.0d"])).is_empty());
        assert!(
            parse_positions(&lines(&["Steve has the following entity data: [a, b, c]"])).is_empty()
        );
    }

    #[test]
    fn parses_scientific_notation() {
        // 坐标极大时服务端可能输出科学计数法
        let parsed = parse_positions(&lines(&[
            "Steve has the following entity data: [1.0E7d, 64.0d, -2.0E7d]",
        ]));
        assert_eq!(parsed[0].1, [1.0e7, 64.0, -2.0e7]);
    }

    #[test]
    fn rotation_with_missing_value_is_skipped() {
        assert!(parse_rotations(&lines(&["Steve has the following entity data: []"])).is_empty());
    }
}
