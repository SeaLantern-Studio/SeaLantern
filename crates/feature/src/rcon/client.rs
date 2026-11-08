//! RCON 协议实现。

use std::time::{Duration, Instant};

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

/// 命令负载的协议上限。
///
/// 协议规定单条命令不超过 1446 字节，超了原版服务端会拒绝或断开。
/// 这里提前拦下，免得把连接写坏。
const MAX_COMMAND_BYTES: usize = 1446;

/// 单条命令累计回显的上限。
///
/// `MAX_BODY` 只管单个包；服务端可以把回显拆成任意多个合法小包，
/// 不设总量上限的话一条命令就能把进程内存吃光。1 MiB 对 `list` /
/// `data get` 这类回显已经很宽松（一千个玩家也就几十 KB）。
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

/// 等待后续响应包的静默窗口。
///
/// 一次命令的回显可能被拆成多个包，而 RCON 协议**没有结束标记**：服务端发完
/// 就不发了，只能靠「一段时间内没有新包」判定结束。Minecraft 的 RCON 是逐包
/// 即时写的，80ms 足够区分。
///
/// 这个近似有两个已知边界，靠 [_`RconClient::command`] 里的逻辑兜住：
/// - 包还没收完（缓冲里有半截帧）时**不**按窗口结束，继续等；
/// - 窗口到期后先非阻塞地把已排队的字节收干净，收到就重新计时。
///
/// 仍然存在理论窗口：某个包在网络上被拆得比 80ms 还散，且中间没有其他包到达。
/// 真要根治只能给命令加哨兵（发一条空命令当结束标记），但空命令在 vanilla /
/// Paper / 各类插件下的回显并不统一，风险大于收益。
const IDLE_WINDOW: Duration = Duration::from_millis(80);

/// 单条命令从发出到收完的总上限，防止服务端持续慢速滴包把连接吊死。
const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);

/// 每次从 socket 读的块大小。
const READ_CHUNK: usize = 8 * 1024;

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
    /// 包类型。认证阶段要据此区分「认证结果」与「普通回显」，
    /// 只比对请求号会把任何同号的包都当成认证通过。
    kind: i32,
    body: String,
}

/// 一个已认证的 RCON 连接。
///
/// 复用连接要比每次调用重新握手划算得多：玩家的位置是秒级轮询的。
pub struct RconClient {
    stream: TcpStream,
    next_id: i32,
    /// 已从 socket 读出、尚未拆成完整帧的字节。
    ///
    /// 必须由连接自己持有，不能放在 `read_packet` 的局部变量里：等待下一个包
    /// 的超时会取消正在进行的读取，而 `read_exact` 被取消时已经读走的字节会
    /// 连同局部缓冲一起丢掉，流就停在帧中间，后续命令把剩下的字节当成新包解析。
    /// `read` 是取消安全的，字节落在 `buf` 里，下一轮接着拆即可。
    buf: Vec<u8>,
    /// 上一次写入是否被中途取消（或写失败）。
    ///
    /// 写操作本身不是取消安全的：调用方在 `write_all` 写了一半时 drop 掉
    /// `command()` 的 future，帧就只发了前半截，之后这条连接上再写什么都会被
    /// 服务端当成前一帧的尾巴。所以写入前置位、写完整帧后再清位；处于置位状态
    /// 时拒绝复用，让调用方重连。
    poisoned: bool,
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

        let mut client = Self {
            stream,
            next_id: 1,
            buf: Vec::new(),
            poisoned: false,
        };

        // 认证用请求号 1；服务端成功时回同样的号，失败时回 -1
        let auth_id = client.take_id();
        client.write_packet(auth_id, TYPE_AUTH, password).await?;

        // 有的实现会在认证响应后再补一个空的 RESPONSE_VALUE，多读一轮兜住
        let mut authed = false;
        for _ in 0..2 {
            let packet = client.next_packet(timeout).await?;
            if packet.id == -1 {
                return Err(RconError::Unauthorized);
            }
            // 请求号相同还必须是「认证结果」类型：只看 id 的话，服务端随手回一个
            // 同号的普通回显包也会被当成认证通过
            if packet.id == auth_id && packet.kind == TYPE_AUTH_RESPONSE {
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
    ///
    /// 三条结束条件，优先级从高到低：
    /// 1. 缓冲里还有收了一半的帧 —— 继续等，绝不按窗口收尾（否则流会停在帧中间）；
    /// 2. 读写出错或包格式非法 —— 上抛，不把截断的回显当成功；
    /// 3. 已收到匹配的回显且窗口内再无数据 —— 正常结束。
    pub async fn command(&mut self, command: &str) -> Result<Vec<String>, RconError> {
        let id = self.take_id();
        self.write_packet(id, TYPE_EXEC, command).await?;

        let deadline = Instant::now() + COMMAND_TIMEOUT;
        let mut lines: Vec<String> = Vec::new();
        let mut matched = false;
        let mut received_bytes = 0usize;

        loop {
            // 先把缓冲里已经完整的帧拆完。上一轮可能一次读进来多个包，
            // 也可能留着上一命令迟到的包。
            while let Some(packet) = self.take_frame()? {
                // 认证阶段的残留包、或别的请求号的包，直接丢弃。
                // 注意 matched 只能在确认请求号匹配之后置位：否则一个无关包
                // 就会启动静默窗口，真正的回显晚到就变成「空结果」。
                if packet.id != id {
                    continue;
                }
                matched = true;
                // 单个包有 MAX_BODY 兜底，但包的数量没有上限：不在这里累计的话，
                // 服务端持续发合法小包就能把内存吃光
                received_bytes += packet.body.len();
                if received_bytes > MAX_RESPONSE_BYTES {
                    return Err(RconError::Protocol(format!(
                        "response exceeded {MAX_RESPONSE_BYTES} bytes"
                    )));
                }
                for line in packet.body.split('\n') {
                    let line = strip_formatting(line).trim_end().to_string();
                    if !line.is_empty() {
                        lines.push(line);
                    }
                }
            }

            // 首包等剩余的整条命令预算，而不是另给一个更短的窗口：
            // 服务器卡住几秒（保存区块、GC）就会误判超时，哪怕总预算还没用完
            let wait = if matched {
                IDLE_WINDOW
            } else {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(RconError::Timeout);
                }
                remaining
            };
            match tokio::time::timeout(wait, self.fill()).await {
                Ok(Ok(read)) if read > 0 => {}
                Ok(Ok(_)) => {
                    // 对端关闭连接。缓冲里还有半截帧说明回显被截断了，必须报错；
                    // 缓冲干净则说明已经收全（有的实现回完就关连接）。
                    if !self.buf.is_empty() {
                        return Err(RconError::Protocol(
                            "connection closed in the middle of a packet".to_string(),
                        ));
                    }
                    if matched {
                        break;
                    }
                    return Err(RconError::Io(std::io::ErrorKind::UnexpectedEof.into()));
                }
                // 读写错误一律上抛：断连和格式错误都不能当成「回显收完了」，
                // 否则调用方会把残缺的玩家列表当成真实快照
                Ok(Err(error)) => return Err(error),
                Err(_) => {
                    // 超时。缓冲里有半截帧说明这个包还没收完，继续等；
                    // 只是还没到下一个包，就先看看 socket 里有没有已经排队的字节。
                    // drain_ready 的 socket 错误同样上抛，不能当成「没有新数据」
                    if !self.buf.is_empty() || self.drain_ready()? {
                        if Instant::now() >= deadline {
                            return Err(RconError::Timeout);
                        }
                        continue;
                    }
                    if matched {
                        break;
                    }
                    return Err(RconError::Timeout);
                }
            }

            if Instant::now() >= deadline {
                return Err(RconError::Timeout);
            }
        }

        Ok(lines)
    }

    /// 取下一个完整包，总时长超过 `wait` 就报超时。
    ///
    /// 用的是**一次性**截止时间而不是每轮重置：否则对端只要不停地滴几个字节，
    /// 每次读都算「有进展」，超时永远不触发，`connect` 会一直挂着。
    async fn next_packet(&mut self, wait: Duration) -> Result<Packet, RconError> {
        let deadline = Instant::now() + wait;
        loop {
            if let Some(packet) = self.take_frame()? {
                return Ok(packet);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(RconError::Timeout);
            }
            match tokio::time::timeout(remaining, self.fill()).await {
                Ok(Ok(read)) if read > 0 => {}
                Ok(Ok(_)) => {
                    return Err(RconError::Io(std::io::ErrorKind::UnexpectedEof.into()));
                }
                Ok(Err(error)) => return Err(error),
                Err(_) => return Err(RconError::Timeout),
            }
        }
    }

    /// 从 socket 读一块追加到 `buf`，返回读到的字节数（0 表示对端关闭）。
    ///
    /// 用 `read` 而不是 `read_exact`：`read` 是取消安全的，被超时打断时
    /// 已经读到的字节留在 `buf` 里，不会让流停在帧中间。
    async fn fill(&mut self) -> Result<usize, RconError> {
        let mut chunk = [0u8; READ_CHUNK];
        let read = self.stream.read(&mut chunk).await.map_err(RconError::Io)?;
        self.buf.extend_from_slice(&chunk[..read]);
        Ok(read)
    }

    /// 非阻塞地把 socket 里已经排队的字节收进缓冲，返回是否收到了新数据。
    ///
    /// `WouldBlock` 表示「暂时没有更多数据」，是正常情况；其它错误是真的读坏了，
    /// 必须上抛 —— 否则调用方会把它当成「这一轮没有新字节」，进而按静默窗口
    /// 把已经收到的部分当成完整回显。
    fn drain_ready(&mut self) -> Result<bool, RconError> {
        let mut chunk = [0u8; READ_CHUNK];
        let mut got_any = false;
        loop {
            match self.stream.try_read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => {
                    self.buf.extend_from_slice(&chunk[..read]);
                    got_any = true;
                }
                // 还有数据没读完就继续，读空了就停
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(RconError::Io(error)),
            }
        }
        Ok(got_any)
    }

    /// 从缓冲里拆出下一个完整包；不够一帧时返回 `None`，不消费任何字节。
    fn take_frame(&mut self) -> Result<Option<Packet>, RconError> {
        if self.buf.len() < 4 {
            return Ok(None);
        }
        // 长度字段包含紧随其后的 8 字节（请求号 + 类型）
        let len = i32::from_le_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]);
        if !(10..=MAX_BODY as i32).contains(&len) {
            return Err(RconError::Protocol(format!("bad packet length {len}")));
        }
        let total = 4 + len as usize;
        if self.buf.len() < total {
            return Ok(None);
        }

        let id = i32::from_le_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]]);
        let kind = i32::from_le_bytes([self.buf[8], self.buf[9], self.buf[10], self.buf[11]]);
        // 服务端只会发这两种：认证结果(2) 与命令回显(0)
        if kind != TYPE_RESPONSE && kind != TYPE_AUTH_RESPONSE {
            return Err(RconError::Protocol(format!("unexpected packet type {kind}")));
        }

        // 负载尾部有两个 0x00，去掉
        let body = String::from_utf8_lossy(&self.buf[12..total])
            .trim_end_matches('\0')
            .to_string();
        self.buf.drain(..total);
        Ok(Some(Packet { id, kind, body }))
    }

    /// 取下一个请求号，跳过 0 与负数（协议里没有特殊含义，但也没必要用）。
    fn take_id(&mut self) -> i32 {
        allocate_id(&mut self.next_id)
    }

    async fn write_packet(&mut self, id: i32, kind: i32, body: &str) -> Result<(), RconError> {
        if self.poisoned {
            return Err(RconError::Protocol(
                "previous write was interrupted; reconnect required".to_string(),
            ));
        }
        // 协议规定命令不超过 1446 字节，超了原版服务端会拒绝或直接断开，
        // 提前拦下比写坏连接好收拾
        if body.len() > MAX_COMMAND_BYTES {
            return Err(RconError::Protocol(format!(
                "command body too long: {} > {MAX_COMMAND_BYTES}",
                body.len()
            )));
        }

        let mut payload = Vec::with_capacity(body.len() + 2);
        payload.extend_from_slice(body.as_bytes());
        payload.push(0);
        payload.push(0);

        let mut frame = Vec::with_capacity(payload.len() + 12);
        frame.extend_from_slice(&((payload.len() + 8) as i32).to_le_bytes());
        frame.extend_from_slice(&id.to_le_bytes());
        frame.extend_from_slice(&kind.to_le_bytes());
        frame.extend_from_slice(&payload);

        // 写不是取消安全的：future 在这儿被 drop 时帧可能只发了一半，
        // 之后的命令会被服务端当成这一帧的尾巴。所以先置毒，写完整帧再解毒。
        self.poisoned = true;
        self.stream.write_all(&frame).await.map_err(RconError::Io)?;
        self.stream.flush().await.map_err(RconError::Io)?;
        self.poisoned = false;
        Ok(())
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
        stream
            .write_all(&encode_frame(id, kind, body))
            .await
            .unwrap();
        stream.flush().await.unwrap();
    }

    /// 编码一帧，便于测试里把同一帧拆成两段发。
    fn encode_frame(id: i32, kind: i32, body: &str) -> Vec<u8> {
        let mut payload = body.as_bytes().to_vec();
        payload.push(0);
        payload.push(0);
        let mut frame = Vec::new();
        frame.extend_from_slice(&((payload.len() + 8) as i32).to_le_bytes());
        frame.extend_from_slice(&id.to_le_bytes());
        frame.extend_from_slice(&kind.to_le_bytes());
        frame.extend_from_slice(&payload);
        frame
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

    /// 回归测试：第二个包被拆成两段、中间间隔远超静默窗口时，不能把半截帧当成收完。
    ///
    /// 故障必须落在**第二个**包上：首包窗口是 3 秒，只有收到第一个包之后静默窗口
    /// 才切到 80ms。旧实现在 80ms 超时时取消 `read_exact`，已读的半截帧随局部
    /// 缓冲一起丢掉并直接结束，于是只返回第一行，连接还停在第二帧中间。
    #[tokio::test]
    async fn packet_arriving_in_two_halves_is_not_truncated() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, _, _) = read_frame(&mut stream).await;
            // 第一个包完整发完，客户端进入 80ms 静默窗口状态
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "Steve has the following entity data: [1.0d, 2.0d, 3.0d]",
            )
            .await;

            // 第二个包拆成两半，中间停 200ms（远大于 80ms）
            let frame = encode_frame(
                exec_id,
                TYPE_RESPONSE,
                "Alex has the following entity data: [4.0d, 5.0d, 6.0d]",
            );
            let split = frame.len() / 2;
            stream.write_all(&frame[..split]).await.unwrap();
            stream.flush().await.unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
            stream.write_all(&frame[split..]).await.unwrap();
            stream.flush().await.unwrap();
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let lines = client.command("x").await.expect("半截帧不该被判成收完");

        assert_eq!(lines.len(), 2, "两个包都要收全，实际 {lines:?}");
        assert!(lines[1].starts_with("Alex"), "第二个包内容不完整: {lines:?}");

        server.await.unwrap();
    }

    /// 回归测试：先到的是别的请求号的包时，不能把它当成响应开始。
    ///
    /// 旧实现先置 `got_any` 再比对请求号，无关包会启动 80ms 静默窗口，
    /// 真正的回显晚到就变成空结果。
    #[tokio::test]
    async fn stale_packet_does_not_end_the_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, _, _) = read_frame(&mut stream).await;
            // 上一个命令迟到的包
            write_frame(&mut stream, exec_id + 7, TYPE_RESPONSE, "leftover").await;
            tokio::time::sleep(Duration::from_millis(200)).await;
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "Steve has the following entity data: [1.0d, 2.0d, 3.0d]",
            )
            .await;
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let lines = client.command("x").await.expect("命令应成功");

        assert_eq!(lines.len(), 1, "无关包不该吃掉真实回显，实际 {lines:?}");
        assert!(lines[0].starts_with("Steve"), "实际 {lines:?}");

        server.await.unwrap();
    }

    /// 回归测试：第二个包没发完就断开，必须报错，不能返回截断的回显。
    ///
    /// 故障同样要落在第二个包上：首包读失败本来就会上抛，只有「已经收到过包」
    /// 之后旧实现才会把断连当成响应正常结束。
    #[tokio::test]
    async fn disconnect_in_the_middle_of_a_packet_is_an_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, _, _) = read_frame(&mut stream).await;
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "Steve has the following entity data: [1.0d, 2.0d, 3.0d]",
            )
            .await;

            let frame = encode_frame(
                exec_id,
                TYPE_RESPONSE,
                "Alex has the following entity data: [4.0d, 5.0d, 6.0d]",
            );
            stream.write_all(&frame[..frame.len() / 2]).await.unwrap();
            stream.flush().await.unwrap();
            stream.shutdown().await.unwrap();
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let result = client.command("x").await;

        assert!(result.is_err(), "半截帧 + 断连必须报错，不能返回截断结果: {result:?}");

        server.await.unwrap();
    }

    /// 回归测试：收到过正常包之后再遇到格式非法的包，错误要上抛。
    ///
    /// 旧实现只判断「有没有收到过包」，格式错误会被当成响应正常结束。
    #[tokio::test]
    async fn malformed_packet_after_a_valid_one_is_an_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, _, _) = read_frame(&mut stream).await;
            write_frame(
                &mut stream,
                exec_id,
                TYPE_RESPONSE,
                "Steve has the following entity data: [1.0d, 2.0d, 3.0d]",
            )
            .await;
            // 类型 7 不在协议允许的范围内
            write_frame(&mut stream, exec_id, 7, "garbage").await;
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let result = client.command("x").await;

        assert!(
            matches!(result, Err(RconError::Protocol(_))),
            "非法包类型要上抛，不能当成回显结束: {result:?}"
        );

        server.await.unwrap();
    }

    /// 认证必须收到「认证结果」类型，光请求号对上不算。
    ///
    /// 旧实现只比 id，服务端回一个同号的普通回显包就会被当成认证通过。
    #[tokio::test]
    async fn auth_requires_auth_response_type() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, kind, _) = read_frame(&mut stream).await;
            assert_eq!(kind, TYPE_AUTH);
            // 请求号对得上，但类型是普通回显(0)而不是认证结果(2)
            write_frame(&mut stream, auth_id, TYPE_RESPONSE, "").await;
            write_frame(&mut stream, auth_id, TYPE_RESPONSE, "").await;
        });

        let result = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2)).await;
        assert!(matches!(result, Err(RconError::Unauthorized)), "同号的普通回显不能算认证成功");

        server.await.unwrap();
    }

    /// 超过协议上限的命令要在写出去之前拦住，不能把连接写坏。
    #[tokio::test]
    async fn oversized_command_is_rejected_before_writing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;
            // 认证之后再不该有别的帧到达
            let mut extra = [0u8; 1];
            let read =
                tokio::time::timeout(Duration::from_millis(300), stream.read(&mut extra)).await;
            assert!(!matches!(read, Ok(Ok(n)) if n > 0), "超长命令不该被写到连接上");
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let result = client.command(&"a".repeat(MAX_COMMAND_BYTES + 1)).await;

        assert!(
            matches!(result, Err(RconError::Protocol(_))),
            "超长命令应报协议错误: {result:?}"
        );

        server.await.unwrap();
    }

    /// 回显累计超过上限要报错，不能无限往内存里堆。
    #[tokio::test]
    async fn oversized_response_is_rejected() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;

            let (exec_id, _, _) = read_frame(&mut stream).await;
            let chunk = "x".repeat(4000);
            // 一直发到对端不再读为止（会被 abort 掉）
            for _ in 0..(MAX_RESPONSE_BYTES / 4000 + 50) {
                write_frame(&mut stream, exec_id, TYPE_RESPONSE, &chunk).await;
            }
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        let result = client.command("x").await;

        assert!(
            matches!(result, Err(RconError::Protocol(_))),
            "超过累计上限应报协议错误: {result:?}"
        );

        server.abort();
    }

    /// 读取被逐字节「滴」的时候，认证不能一直挂着。
    ///
    /// 旧实现对每次成功读取都重置整个超时，只要对端慢速滴包就永远不超时。
    #[tokio::test]
    async fn trickled_auth_frame_times_out() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_frame(&mut stream).await;
            // 一个合法但永远发不完的帧头
            for _ in 0..100 {
                if stream.write_all(&[0u8]).await.is_err() {
                    return;
                }
                let _ = stream.flush().await;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        let started = Instant::now();
        let result =
            RconClient::connect(&addr.to_string(), "secret", Duration::from_millis(300)).await;
        let elapsed = started.elapsed();

        assert!(result.is_err(), "滴包不该让 connect 成功");
        assert!(elapsed < Duration::from_secs(2), "必须按总时长超时，实际用了 {elapsed:?}");

        server.abort();
    }

    /// 写入被取消（或写失败）之后连接不能继续复用。
    #[tokio::test]
    async fn poisoned_client_refuses_reuse() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (auth_id, _, _) = read_frame(&mut stream).await;
            write_frame(&mut stream, auth_id, TYPE_AUTH_RESPONSE, "").await;
            // 保持连接打开，让客户端有机会去写
            tokio::time::sleep(Duration::from_millis(500)).await;
        });

        let mut client = RconClient::connect(&addr.to_string(), "secret", Duration::from_secs(2))
            .await
            .expect("认证应成功");
        // 模拟「上一次写入被 drop 掉」留下的状态
        client.poisoned = true;

        let result = client.command("list").await;

        assert!(
            matches!(result, Err(RconError::Protocol(_))),
            "处于毒化状态的连接必须拒绝复用: {result:?}"
        );

        server.abort();
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
