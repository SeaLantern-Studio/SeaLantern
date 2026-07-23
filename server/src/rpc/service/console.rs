//! 服务器控制台输入的宿主能力端口。

/// 由受管服务器运行时返回的控制台输入失败类别。
///
/// 宿主实现必须在向任何子进程写入前验证该实例的 stdin 确实属于受管服务端进程。脚本、
/// shell 或自定义启动包装进程的 stdin 必须返回 [`Self::InputUnavailable`]，而不能被视为
/// 可写控制台。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleCommandServiceError {
    /// 指定的服务器实例不存在。
    InstanceNotFound,
    /// 实例未运行，或其 stdin 不属于已验证的服务端进程。
    InputUnavailable,
    /// 已验证的服务端控制台暂时无法接收输入。
    DeliveryFailed,
}

/// 向已验证服务器控制台发送单行命令的宿主能力。
///
/// 该端口不接受 Tauri、HTTP 或插件运行时类型。实现必须保证错误不会携带命令正文、凭据
/// 或主机路径；底层失败详情应仅写入受控的宿主日志。
pub trait ConsoleCommandService: Send + Sync {
    /// 向指定实例的受管服务端进程发送一条已验证命令。
    fn send_console_command(
        &self,
        instance_id: &str,
        command: &str,
    ) -> Result<(), ConsoleCommandServiceError>;
}
