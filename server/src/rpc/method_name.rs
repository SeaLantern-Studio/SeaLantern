//! RPC 方法的稳定标识和 HTTP 路径派生规则。

use std::fmt;

const HTTP_RPC_PREFIX: &str = "/api/rpc";

/// 传输无关的 RPC 方法标识。
///
/// 标识固定使用小写点分的 `domain.resource.action` 形式，例如
/// `server.console.send`。Rust 实现仍可使用 idiomatic 的模块、类型和函数命名；前端
/// 适配器则可直接以此标识组织调用，并由 [`Self::http_path`] 获得一致的 Axum 路径。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RpcMethodName(&'static str);

impl RpcMethodName {
    /// 使用编译期校验的静态方法标识创建名称。
    ///
    /// 每个段必须以小写 ASCII 字母开头，后续只能包含小写 ASCII 字母或数字；段之间仅
    /// 能用一个 `.` 分隔。这使标识可以无歧义映射为 HTTP 路径。
    pub const fn new(value: &'static str) -> Self {
        assert!(Self::is_valid(value), "invalid RPC method name");
        Self(value)
    }

    /// 判断动态获得的标识是否符合公共命名规则。
    pub const fn is_valid(value: &str) -> bool {
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return false;
        }

        let mut index = 0;
        let mut needs_segment_start = true;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte == b'.' {
                if needs_segment_start {
                    return false;
                }
                needs_segment_start = true;
            } else if needs_segment_start {
                if !is_ascii_lowercase_letter(byte) {
                    return false;
                }
                needs_segment_start = false;
            } else if !is_ascii_lowercase_letter(byte) && !is_ascii_digit(byte) {
                return false;
            }
            index += 1;
        }

        !needs_segment_start
    }

    /// 返回稳定的传输无关方法标识。
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// 派生 Axum 应注册的 POST 路径。
    ///
    /// HTTP 只承载 RPC 方法，不混入旧的动态 `/api/{command}` 表。请求和响应 JSON 字段
    /// 的 camelCase 规则由未来的 Axum/Tauri 适配器 DTO 负责。
    pub fn http_path(self) -> String {
        let mut path = String::with_capacity(HTTP_RPC_PREFIX.len() + self.0.len() + 1);
        path.push_str(HTTP_RPC_PREFIX);
        path.push('/');
        for character in self.0.chars() {
            path.push(if character == '.' { '/' } else { character });
        }
        path
    }
}

impl fmt::Debug for RpcMethodName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RpcMethodName")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for RpcMethodName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

const fn is_ascii_lowercase_letter(byte: u8) -> bool {
    byte >= b'a' && byte <= b'z'
}

const fn is_ascii_digit(byte: u8) -> bool {
    byte >= b'0' && byte <= b'9'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_a_stable_http_path_from_the_canonical_name() {
        let method = RpcMethodName::new("server.console.send");

        assert_eq!(method.as_str(), "server.console.send");
        assert_eq!(method.http_path(), "/api/rpc/server/console/send");
    }

    #[test]
    fn accepts_lowercase_dotted_domain_resource_action_names() {
        assert!(RpcMethodName::is_valid("server.instance.create"));
        assert!(RpcMethodName::is_valid("plugin.v2.enable"));
    }

    #[test]
    fn rejects_ambiguous_or_transport_unsafe_names() {
        assert!(!RpcMethodName::is_valid(""));
        assert!(!RpcMethodName::is_valid("server..send"));
        assert!(!RpcMethodName::is_valid("server.console."));
        assert!(!RpcMethodName::is_valid("server.console.send-command"));
        assert!(!RpcMethodName::is_valid("Server.console.send"));
        assert!(!RpcMethodName::is_valid("server.2console.send"));
    }
}
