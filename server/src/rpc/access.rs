//! RPC 方法的权限契约。

use std::collections::BTreeSet;
use std::fmt;

use super::RpcMethodName;

/// 可由受信任适配器授予的稳定 RPC 权限。
///
/// 权限名称遵循与 RPC 方法相同的小写点分规则，但它是独立的授权契约；调用方法本身不等于
/// 调用方已获授权限。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RpcPermission(&'static str);

impl RpcPermission {
    /// 使用编译期校验的静态权限名称创建权限。
    pub const fn new(value: &'static str) -> Self {
        assert!(RpcMethodName::is_valid(value), "invalid RPC permission name");
        Self(value)
    }

    /// 返回稳定的机器可读权限名称。
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Debug for RpcPermission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RpcPermission")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for RpcPermission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// 当前请求已经通过身份验证和授权后获得的权限集合。
///
/// 此类型仅能由受信任的进程内适配器或后续授权服务构建，绝不能反序列化自 HTTP、Tauri
/// 或插件请求的参数。默认值不包含任何权限。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RpcAccess {
    permissions: BTreeSet<RpcPermission>,
    /// 是否授予全部权限（受信任调用方的哨兵标记）。
    ///
    /// 一旦为真，`allows` 恒真；仅 [`Self::allow_all`] 会设置。
    unbounded: bool,
}

impl RpcAccess {
    /// 构建不授予任何权限的访问集合。
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// 使用已由受信任边界决定的权限构建访问集合。
    pub fn allow(permissions: impl IntoIterator<Item = RpcPermission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
            unbounded: false,
        }
    }

    /// 构建授予全部权限的访问集合。
    ///
    /// 仅供受信任的进程内调用方（如桌面端本地宿主、插件运行时）使用；
    /// 网络或不受信任边界必须显式枚举权限，不能调用此方法。
    pub fn allow_all() -> Self {
        // 以 BTreeSet 无法表达"全部"，这里用哨兵语义：构造一个包含所有已知权限
        // 的集合成本过高且无法随新权限自动扩展，因此实现为允许一切（见 `allows`）。
        Self::deny_all().with_unbounded_permissions()
    }

    fn with_unbounded_permissions(mut self) -> Self {
        self.unbounded = true;
        self
    }

    /// 判断当前调用是否已获授指定权限。
    pub fn allows(&self, permission: RpcPermission) -> bool {
        self.unbounded || self.permissions.contains(&permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONSOLE_SEND: RpcPermission = RpcPermission::new("server.console.send");
    const INSTANCE_CREATE: RpcPermission = RpcPermission::new("server.instance.create");

    #[test]
    fn denies_permissions_by_default() {
        assert!(!RpcAccess::deny_all().allows(CONSOLE_SEND));
    }

    #[test]
    fn grants_only_explicit_permissions() {
        let access = RpcAccess::allow([CONSOLE_SEND]);

        assert!(access.allows(CONSOLE_SEND));
        assert!(!access.allows(INSTANCE_CREATE));
    }

    #[test]
    fn allow_all_grants_every_permission() {
        let access = RpcAccess::allow_all();

        assert!(access.allows(CONSOLE_SEND));
        assert!(access.allows(INSTANCE_CREATE));
    }
}
