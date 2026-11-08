//! RCON 协议实现。

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// 认证包类型（客户端 → 服务端）。
const TYPE_AUTH: i32 = 3;
/// 认证响应类型（服务端 → 客户端）。
///
/// 协议里这个值和「执行命令」共用 2，方向不同含义不同：客户端发 2 是执行命令，
/// 服务端发 2 是认证结果。读包时必须放行它，否则认证永远失败。
const TYPE_AUTH_RESPONSE: i32 = 2;
/// 执行命令包类型（客户端 → 服务端）。
const TYPE_EXEC: i32 = 2;
/// 命令回显包类型（服务端 → 客户端）。
const TYPE_RESPONSE: i32 = 0;

/// 单包负载上限。协议层规定命令最长 1446 字节；回显可能远大于此
/// （例如 `list` 在人多时），给到 4 MiB 足够且能防住异常长度。
const MAX_BODY: usize = 4 * 1024 * 1024;

/// 等待后续响应包的静默窗口。
///
/// 一次命令的回显可能被拆成多个包，服务端发完就不再发；靠「一段时间内没有
/// 新包」判定结束。Minecraft 的 RCON 是逐包即时写的，50ms 足够区分。
const IDLE_WINDOW: Duration = Duration::from_millis(80);

/// RCON 操作错误。
#[derive(Debug)]
pub enum RconError {
    /// TCP 连接失败（多为服务器未运行或端口不通）。
    Connect(std::io::Error),
    /// 连接建立后收发失败。
    Io(std::io::Error),
    /// 认证包没有在超时内得到响应。
    Timeout,
    /// 密码错误。
    Unauthorized,
    /// 协议层异常（长度非法、包类型不符等）。
    Protocol(String),
}

impl std::fmt::Display for RconError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(error) => write!(formatter, "rcon connect failed: {error}"),
            Self::Io(error) => write!(formatter, "rcon io failed: {error}"),
            Self::Timeout => formatter.write_str("rcon timed out"),
            Self::Unauthorized => formatter.write_str("rcon authentication failed"),
            Self::Protocol(reason) => write!(formatter, "rcon protocol error: {reason}"),
        }
    }
}

impl std::error::Error for RconError {}

/// 解析后的响应包。
struct Packet {
    id: i32,
    body: String,
}

/// 一个已认证的 RCON 连接。
///
/// 复用连接要比每次调用重新握手划算得多：玩家的位置是秒级轮询的。
pub struct RconClient {
    stream: TcpStream,
    next_id: i32,
}

impl RconClient {
    /// 建立连接并完成认证。
    ///
    /// `timeout` 同时用于连接与认证，避免端口被防火墙吞包时长时间挂住。
    pub async fn connect(addr: &str, password: &str, timeout: Duration) -> Result<Self, RconError> {
        let stream = tokio::time::timeout(timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| RconError::Timeout)?
            .map_err(RconError::Connect)?;
        let _ = stream.set_nodelay(true);

        let mut client = Self { stream, next_id: 1 };

        // 认证用请求号 1；服务端成功时回同样的号，失败时回 -1
        let auth_id = client.take_id();
        client.write_packet(auth_id, TYPE_AUTH, password).await?;

        // 有的实现会在认证响应后再补一个空的 RESPONSE_VALUE，多读一轮兜住
        let mut authed = false;
        for _ in 0..2 {
            let packet = match tokio::time::timeout(timeout, client.read_packet()).await {
                Ok(result) => result?,
                Err(_) => return Err(RconError::Timeout),
            };
            if packet.id == -1 {
                return Err(RconError::Unauthorized);
            }
            if packet.id == auth_id {
                authed = true;
                break;
            }
        }
        if !authed {
            return Err(RconError::Unauthorized);
        }

        Ok(client)
    }

    /// 执行一条命令，返回回显按行拆分后的文本。
    ///
    /// 服务端可能把长回显拆成多个包，这里按静默窗口合并；命令成功但没有
    /// 任何回显时返回空 `Vec`（例如 `execute if` 条件不成立）。
    pub async fn command(&mut self, command: &str) -> Result<Vec<String>, RconError> {
        let id = self.take_id();
        self.write_packet(id, TYPE_EXEC, command).await?;

        let mut lines: Vec<String> = Vec::new();
        let mut got_any = false;
        loop {
            let wait = if got_any {
                IDLE_WINDOW
            } else {
                Duration::from_secs(3)
            };
            let packet = match tokio::time::timeout(wait, self.read_packet()).await {
                Ok(Ok(packet)) => packet,
                // 首包都等不到才算失败；收过包之后的超时就是「发完了」
                Ok(Err(error)) => {
                    if got_any {
                        break;
                    }
                    return Err(error);
                }
                Err(_) => {
                    if got_any {
                        break;
                    }
                    return Err(RconError::Timeout);
                }
            };
            got_any = true;

            // 认证阶段的残留包、或别的请求号的包，直接丢弃
            if packet.id != id {
                continue;
            }
            for line in packet.body.split('\n') {
                let line = strip_formatting(line).trim_end().to_string();
                if !line.is_empty() {
                    lines.push(line);
                }
            }
        }

        Ok(lines)
    }

    /// 取下一个请求号，跳过 0 与负数（协议里没有特殊含义，但也没必要用）。
    fn take_id(&mut self) -> i32 {
        allocate_id(&mut self.next_id)
    }

    async fn write_packet(&mut self, id: i32, kind: i32, body: &str) -> Result<(), RconError> {
        let mut payload = Vec::with_capacity(body.len() + 2);
        payload.extend_from_slice(body.as_bytes());
        payload.push(0);
        payload.push(0);

        let mut frame = Vec::with_capacity(payload.len() + 12);
        frame.extend_from_slice(&((payload.len() + 8) as i32).to_le_bytes());
        frame.extend_from_slice(&id.to_le_bytes());
        frame.extend_from_slice(&kind.to_le_bytes());
        frame.extend_from_slice(&payload);

        self.stream.write_all(&frame).await.map_err(RconError::Io)?;
        self.stream.flush().await.map_err(RconError::Io)
    }

    async fn read_packet(&mut self) -> Result<Packet, RconError> {
        let mut len_buf = [0u8; 4];
        self.stream
            .read_exact(&mut len_buf)
            .await
            .map_err(RconError::Io)?;
        let len = i32::from_le_bytes(len_buf);
        // 长度字段包含自己的 8 字节（请求号 + 类型）
        if !(10..=MAX_BODY as i32).contains(&len) {
            return Err(RconError::Protocol(format!("bad packet length {len}")));
        }

        let mut buf = vec![0u8; len as usize];
        self.stream
            .read_exact(&mut buf)
            .await
            .map_err(RconError::Io)?;

        let id = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let kind = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        // 服务端只会发这两种：认证结果(2) 与命令回显(0)
        if kind != TYPE_RESPONSE && kind != TYPE_AUTH_RESPONSE {
            return Err(RconError::Protocol(format!("unexpected packet type {kind}")));
        }

        // 负载尾部有两个 0x00，去掉
        let body = String::from_utf8_lossy(&buf[8..])
            .trim_end_matches('\0')
            .to_string();
        Ok(Packet { id, body })
    }

    /// 主动断开。失败无所谓，对端多半已经先关了。
    pub async fn close(mut self) {
        let _ = self.stream.shutdown().await;
    }
}

/// 从计数器里取一个请求号，并把计数器推到下一个合法值。
///
/// 单独抽出来是为了能脱离网络连接测试回绕行为（`RconClient` 持有 `TcpStream`，
/// 而 `TcpStream::from_std` 需要在 tokio 运行时里才能构造）。
fn allocate_id(current: &mut i32) -> i32 {
    let id = *current;
    *current = current.wrapping_add(1).max(1);
    id
}

/// 去掉 Minecraft 的 `§x` 颜色控制码。
///
/// 服务器控制台一般不带颜色，但插件（聊天前缀、计分板队伍）可能塞进来，
/// 不清理会让玩家名对不上。
fn strip_formatting(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '§' {
            // 跳过格式码本身。`§x` 后面跟 6 位十六进制属于同一个码，
            // 逐个字符丢弃也无副作用（都是合法十六进制），这里一并丢掉。
            let _ = chars.next();
            continue;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_color_codes() {
        assert_eq!(strip_formatting("§aSteve§r"), "Steve");
        assert_eq!(strip_formatting("no codes"), "no codes");
        assert_eq!(strip_formatting("§"), "");
    }

    #[test]
    fn request_ids_never_collide_with_reserved_values() {
        let mut next = i32::MAX;
        assert_eq!(allocate_id(&mut next), i32::MAX);
        // wrapping 之后是 i32::MIN，必须被抬回正数
        assert_eq!(allocate_id(&mut next), 1);
        assert_eq!(allocate_id(&mut next), 2);
    }
}

/// 协议层端到端测试：在本地起一个假的 RCON 服务端，验证握手与收发。
///
/// 这些用例存在的意义：协议字段错一个字节（长度前缀、类型码、尾部补零）
/// 单元测试是看不出来的，只有真的收发一次才会暴露。
#[cfg(test)]
mod protocol_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    /// 按协议读一帧，返回 (请求号, 类型, 负载)。
    async fn read_frame(stream: &mut TcpStream) -> (i32, i32, String) {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = i32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await.unwrap();
        let id = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let kind = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let body = String::from_utf8_lossy(&buf[8..])
            .trim_end_matches('\0')
            .to_string();
        (id, kind, body)
    }

    /// 按协议写一帧。
    async fn write_frame(stream: &mut TcpStream, id: i32, kind: i32, body: &str) {
        let mut payload = body.as_bytes().to_vec();
        payload.push(0);
        payload.push(0);
        let mut frame = Vec::new();
        frame.extend_from_slice(&((payload.len() + 8) as i32).to_le_bytes());
        frame.extend_from_slice(&id.to_le_bytes());
        frame.extend_from_slice(&kind.to_le_bytes());
        frame.extend_from_slice(&payload);
        stream.write_all(&frame).await.unwrap();
        stream.flush().await.unwrap();
    }

    #[tokio::test]
    async fn authenticates_then_executes_command() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            let (auth_id, kind, body) = read_frame(&mut stream).await;
            assert_eq!(kind, TYPE_AUTH, "首个包必须是认证");
            assert_eq!(body, "secret");
            // 成功认证：回同样的请求号，类型 2
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, kind, body) = read_frame(&mut stream).await;
            assert_eq!(kind, TYPE_EXEC);
            assert_eq!(body, "list");
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "There are 1 of a max of 20 players online",
            )
            .await;
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let lines = client.command("list").await.expect("命令应成功");
        assert_eq!(lines, vec!["There are 1 of a max of 20 players online"]);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn wrong_password_yields_unauthorized() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (_, kind, _) = read_frame(&mut stream).await;
            assert_eq!(kind, TYPE_AUTH);
            // 失败：请求号回 -1
            write_frame(&mut stream, -1, TYPE_AUTH_RESPONSE, "").await;
        });

        let result = RconClient::connect(&addr.to_string(), "nope", Duration::from_secs(2)).await;
        assert!(matches!(result, Err(RconError::Unauthorized)));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn merges_multi_packet_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, _, _) = read_frame(&mut stream).await;
            // 长回显被拆成两包，客户端要合并
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "Steve has the following entity data: [1.0d, 2.0d, 3.0d]",
            )
            .await;
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "Alex has the following entity data: [4.0d, 5.0d, 6.0d]",
            )
            .await;
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let lines = client.command("x").await.expect("命令应成功");
        assert_eq!(lines.len(), 2, "两包都要收下");
        assert!(lines[1].starts_with("Alex"));

        server.await.unwrap();
    }

    #[tokio::test]
    async fn connect_to_closed_port_is_reported() {
        // 先占一个端口再关掉，保证这个端口此刻没人监听
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let result =
            RconClient::connect(&addr.to_string(), "secret", Duration::from_millis(300)).await;
        assert!(result.is_err(), "端口没人监听时必须报错，不能挂在那里");
    }
}
