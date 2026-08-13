//! 服务器进程输出流的逐行读取与解码原语。
//!
//! 提供将 [`TerminalOutput`] 等字节流按行读取、跨平台解码的基础能力：
//! 读取与解码均为同步纯函数，执行上下文（如 `spawn_blocking`）由上层
//! 调用方决定，本模块不启动任何线程或任务。

use std::io::{self, BufRead, Read};

/// 将进程输出字节解码为文本。
///
/// 优先按 UTF-8 解码；失败时（常见于 Windows 控制台输出 GBK 编码）
/// 尝试按 GBK 解码，仍失败则使用替换字符兜底。
pub fn decode_output_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }

    #[cfg(target_os = "windows")]
    {
        let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
        decoded.into_owned()
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// 从读取器逐行读取并解码，对每一行调用 `on_line`。
///
/// 以 `\n` 分割行，去除行尾 `\r`，跳过空白行。返回读取器到达 EOF 或
/// 读取失败时的错误；调用方持有多行时无需处理行缓冲。
pub fn read_output_lines<R: Read>(reader: R, mut on_line: impl FnMut(&str)) -> io::Result<()> {
    let mut reader = io::BufReader::new(reader);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer) {
            Ok(0) => return Ok(()),
            Ok(_) => {
                let mut line = decode_output_bytes(&buffer);
                line = line.trim_end_matches(['\r', '\n']).to_string();
                if line.trim().is_empty() {
                    continue;
                }
                on_line(&line);
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{decode_output_bytes, read_output_lines};

    #[test]
    fn decodes_utf8_output_as_is() {
        assert_eq!(decode_output_bytes("你好，世界".as_bytes()), "你好，世界");
    }

    #[test]
    fn decodes_non_utf8_with_lossy_fallback() {
        // 0xFF 不是合法 UTF-8，也不会是 GBK 的单字节序列，应走替换字符。
        let decoded = decode_output_bytes(&[0x61, 0xFF, 0x62]);
        assert!(decoded.contains('�'));
    }

    #[test]
    fn reads_lines_splitting_on_newline_and_trimming_carriage_return() {
        let input = "line one\r\nline two\n\n  \nline three\n";
        let mut lines = Vec::new();
        read_output_lines(Cursor::new(input), |line| lines.push(line.to_string()))
            .expect("read lines");

        assert_eq!(lines, ["line one", "line two", "line three"]);
    }
}
