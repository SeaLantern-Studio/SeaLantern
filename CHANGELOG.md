# 日志

## 概述

新增了命令行界面(CLI)支持、安全性增强和性能优化。本版本引入了 20 个 CLI 命令，实现输入验证机制，并采用环形缓冲区技术优化日志处理性能

---

## 新功能

### 命令行界面 (CLI) - 20个命令

SeaLantern 现在支持完整的命令行操作模式，允许用户在没有图形界面的情况下管理 Minecraft 服务器。

#### 启动方式

```bash
# GUI 模式（默认）- 启动图形界面
sea-lantern.exe

# CLI 模式 - 在命令后添加 --cli 参数
sea-lantern.exe --cli list
sea-lantern.exe --cli start my-server
sea-lantern.exe --cli -f json list  # JSON 格式输出
```

#### 命令详解

| 命令 | 语法 | 功能描述 | 示例 |
| **list** | `sea-lantern list [-r, --running]` | 列出所有已创建的服务器实例。使用 `-r` 参数仅显示运行中的服务器。 | `sea-lantern list`<br>`sea-lantern list -r` |
| **start** | `sea-lantern start <ID> [-w, --wait]` | 启动指定的 Minecraft 服务器。使用 `-w` 参数等待服务器完全启动后返回。 | `sea-lantern start my-server`<br>`sea-lantern start my-server -w` |
| **stop** | `sea-lantern stop <ID> [-f, --force]` | 停止指定服务器。默认发送 `stop` 命令优雅关闭，使用 `-f` 强制终止进程。 | `sea-lantern stop my-server`<br>`sea-lantern stop my-server -f` |
| **restart** | `sea-lantern restart <ID> [-d, --delay <秒>]` | 重启指定服务器。默认延迟 3 秒后重启，可自定义延迟时间。 | `sea-lantern restart my-server`<br>`sea-lantern restart my-server -d 5` |
| **send** | `sea-lantern send <ID> <命令>` | 向服务器控制台发送 Minecraft 命令。 | `sea-lantern send my-server op player1`<br>`sea-lantern send my-server gamemode creative` |
| **status** | `sea-lantern status <ID>` | 显示服务器详细状态，包括运行状态、PID、内存占用、在线玩家数等。 | `sea-lantern status my-server` |
| **info** | `sea-lantern info <ID>` | 显示服务器配置信息：Java 版本、分配内存、服务端类型、世界设置等完整配置。 | `sea-lantern info my-server` |
| **logs** | `sea-lantern logs <ID> [-l, --lines <行数>] [-e, --errors]` | 查看服务器日志。默认显示最后 50 行，可指定行数。使用 `-e` 仅显示错误日志。 | `sea-lantern logs my-server`<br>`sea-lantern logs my-server -l 100`<br>`sea-lantern logs my-server -e` |
| **monitor** | `sea-lantern monitor <ID> [-t, --timestamp]` | 实时监控服务器日志输出（类似 `tail -f`）。使用 `-t` 显示时间戳。按 Ctrl+C 退出。 | `sea-lantern monitor my-server`<br>`sea-lantern monitor my-server -t` |
| **rename** | `sea-lantern rename <旧名> <新名>` | 重命名服务器实例。注意：这会同时重命名服务器目录。 | `sea-lantern rename old-name new-name` |
| **delete** | `sea-lantern delete <ID> [-y, --yes]` | 删除服务器实例及其所有数据。默认需要确认，使用 `-y` 跳过确认。**注意：此操作不可恢复！** | `sea-lantern delete my-server`<br>`sea-lantern delete my-server -y` |
| **stop-all** | `sea-lantern stop-all [-y, --yes]` | 停止所有正在运行的服务器。用于批量关闭或关机前安全停止。 | `sea-lantern stop-all`<br>`sea-lantern stop-all -y` |
| **running** | `sea-lantern running` | 仅列出当前正在运行的服务器，显示简洁视图包括运行时间和端口。 | `sea-lantern running` |
| **search** | `sea-lantern search <关键词>` | 在服务器列表中搜索匹配的服务器名称、核心类型或 MC 版本。 | `sea-lantern search paper`<br>`sea-lantern search 1.20` |
| **exec** | `sea-lantern exec <目标> <命令>` | 批量执行命令。目标可以是逗号分隔的服务器 ID，或使用 `all` 表示所有运行中的服务器。 | `sea-lantern exec all save-all`<br>`sea-lantern exec server1,server2 say Hello` |
| **export** | `sea-lantern export [-o, --output <文件>]` | 导出服务器列表配置为 JSON 格式。不指定输出文件则打印到控制台。 | `sea-lantern export`<br>`sea-lantern export -o servers.json` |
| **watch** | `sea-lantern watch [-i, --interval <秒>]` | 实时监控所有服务器状态变化，显示仪表盘界面。默认 2 秒刷新。按 Ctrl+C 退出。 | `sea-lantern watch`<br>`sea-lantern watch -i 5` |
| **create** | `sea-lantern create <名称> [选项]` | 创建新的服务器实例（概念）。 |开发中|
| **import** | `sea-lantern import <路径> [名称]` | 从外部路径导入现有服务器（概念）。 |开发中|
| **help** | `sea-lantern help [命令]` | 显示帮助信息。可查看特定命令的详细用法。 | `sea-lantern help`<br>`sea-lantern help start` |

#### 输出格式

CLI 支持 `text`（默认）和 `json` 两种输出格式，通过 `-f` 或 `--format` 参数指定：

```bash
# 文本格式（默认）- 适合人类阅读
sea-lantern list

# JSON 格式 - 适合脚本解析
sea-lantern -f json list
sea-lantern --format json status my-server
```

JSON 输出示例：
```json
[
  {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "name": "my-server",
    "core_type": "paper",
    "mc_version": "1.20.4",
    "port": 25565,
    "path": "C:\\Servers\\my-server"
  }
]
```

#### 服务器 ID 解析

大多数命令接受服务器 ID 或名称作为参数。CLI 支持智能匹配：

- **精确 ID 匹配**: 使用完整 UUID
- **精确名称匹配**: 使用服务器的完整名称
- **模糊名称匹配**: 使用名称的部分字符（如果只匹配到一个）

```bash
# 以下三种方式都可以定位到名为 "my-survival-server" 的服务器
sea-lantern start a1b2c3d4-e5f6-7890-abcd-ef1234567890  # 精确 ID
sea-lantern start my-survival-server                      # 精确名称
sea-lantern start survival                                # 模糊匹配
```

---

## 安全增强

### 1. 输入验证机制

添加了输入验证机制，防止多种常见攻击向量：

#### 路径遍历防护

阻止攻击者通过 `../` 等路径遍历字符访问服务器目录外的敏感文件。（基于最近的经历，所以添加了这个）

**实现位置**: [`server_manager.rs`](src-tauri/src/services/server_manager.rs)

**防护策略**:
- 检测并阻止 `../` 和 `..\\` 路径遍历序列
- 规范化路径比较，防止编码绕过
- 验证最终路径是否在允许的目录范围内

#### 命令注入检测

过滤可能导致命令注入的危险字符和操作符。

**防护策略**:
- 检测 shell 命令连接符: `&&`, `||`, `|`, `;`
- 检测命令替换语法: `` ` ``, `$()`
- 检测输出重定向: `>>`

#### Windows 保留名阻断

防止使用 Windows 系统保留名称创建服务器，这些名称可能导致系统异常（可能）

**保留名称列表**:
- 设备名: `CON`, `PRN`, `AUX`, `NUL`
- 串口: `COM1` - `COM9`
- 并口: `LPT1` - `LPT9`

### 2. 名称验证规则

服务器名称现在必须满足以下规则：

| 规则 | 说明 | 示例 |
| 非空 | 名称不能为空或仅包含空白字符 | ❌ `""`, `"   "` |
| 长度限制 | 最大 64 个字符 | ❌ `"a".repeat(65)` |
| 非法字符 | 不能包含 `/\:*?"<>|` 和空字符 | ❌ `"my/server"`, `"server?name"` |
| 点号限制 | 不能以点开头或结尾 | ❌ `".server"`, `"server."` |
| 尾部空格 | 不能以空格结尾 | ❌ `"server "` |
| 保留名 | 不能使用 Windows 保留名称 | ❌ `"CON"`, `"NUL"` |

---

## 性能优化

### 1. 环形缓冲区日志缓存

日志处理是 Minecraft 服务器管理器的高频操作。现在采用环形缓冲区（Ring Buffer）重构日志缓存，实现 O(1) 时间复杂度的读写操作。

**实现位置**: [`server_log_pipeline.rs`](src-tauri/src/services/server_log_pipeline.rs)

#### 设计原理

环形缓冲区是一种固定大小的数据结构，通过维护头尾指针实现循环写入：

```
初始状态:
[None, None, None, None, None]
 head
 tail

写入 3 条日志后:
[Log1, Log2, Log3, None, None]
 head           tail

写入 5 条日志后（满）:
[Log1, Log2, Log3, Log4, Log5]
 head                 tail
（tail 环绕回到 head）

读取并清空:
[None, None, None, None, None]
 head
 tail
```
#### 批处理参数

```rust
const LOG_BATCH_SIZE: usize = 128;           // 每批最多 128 条日志
const LOG_FLUSH_INTERVAL_MS: u64 = 50;       // 50ms 批处理窗口
```

这些参数平衡了吞吐量和延迟：
- 高频输出时自动批量提交，减少事务次数
- 低频输出时最多等待 50ms 即可刷盘
- 突发流量时避免无限增长（固定大小缓冲区）

### 2. 异步日志写入器架构

日志系统采用生产者-消费者模式，将日志写入与主线程解耦：

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   主线程/其他    │────▶│    消息队列     │────▶│  Writer 线程    │
│  append_log()   │     │  mpsc::channel  │     │  SQLite 写入    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
      非阻塞                   缓冲                    批量提交
```

**优势**:
- 日志输出不阻塞服务器进程的 stdout/stderr 消费
- 突发日志时自动缓冲，平滑 I/O 峰值
- 写入失败不影响主服务流程

### 3. 虚拟滚动优化

控制台组件实现了虚拟滚动，大幅减少 DOM 节点数量：

**实现位置**: [`ConsoleInput.vue`](src/components/console/ConsoleInput.vue)

**优化效果**:
- 仅渲染可见区域的日志行
- 支持数万行日志流畅滚动
- 内存占用恒定，不随日志量增长

---

## 技术架构

### CLI 模式切换

程序启动时通过 `--cli` 参数判断运行模式：

```rust
// main.rs
let args: Vec<String> = std::env::args().collect();
if args.iter().any(|arg| arg == "--cli") {
    // 进入 CLI 模式，解析命令并执行
    run_cli_mode(args);
} else {
    // 启动 Tauri GUI 应用
    tauri::Builder::default().run(tauri::generate_context!()).unwrap();
}
```

### 参数解析

使用 `clap` crate 进行参数解析，支持子命令和全局选项：

```rust
#[derive(Parser)]
#[command(name = "sea-lantern")]
#[command(about = "Minecraft 服务器管理器", long_about = None)]
struct Cli {
    /// 输出格式 (text/json)
    #[arg(short, long, global = true, default_value = "text")]
    format: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List { /* ... */ },
    Start { /* ... */ },
    // ... 其他命令
}
```

### 参数过滤技巧

由于 `--cli` 是模式切换标志而非 clap 参数，需要在解析前过滤：

```rust
// 过滤掉 --cli 标志，避免 clap 报错
let filtered_args: Vec<String> = std::env::args()
    .filter(|arg| arg != "--cli")
    .collect();
let cli = Cli::parse_from(filtered_args);
```

---
## 已知问题

1. **create/import 命令**: CLI 版创建和导入服务器功能正在开发中（lan）
2. **颜色输出**: Windows CMD 部分颜色代码可能显示异常
3. **中文路径**: 某些情况下中文路径可能需要 UTF-8 编码
