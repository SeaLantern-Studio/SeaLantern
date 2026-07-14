# Sea Lantern 服务端 CPU 亲和性与 JVM Flags 临时草案

> 状态：临时草案
>
> 目的：先收敛配置模型、运行时语义与实现顺序，暂不进入代码实现。
>
> 备注：该文档仅作为 `docs/` 下的临时设计稿，不应作为本阶段提交目标。

## 背景

Sea Lantern 当前已经具备两条 Minecraft Java 服务端运行路径：

- 本地 Java 进程运行
- `itzg/minecraft-server` Docker 运行

当前模型已经存在 `jvm_args`，但还缺少两类能力：

- 可配置的 CPU 亲和性或 CPU 核心限制
- 可选择的 JVM flags / 优化 flags 预设

这两类能力容易被混淆，但它们不是同一个东西：

- `CPU affinity` 解决的是“进程实际允许运行在哪些 CPU core 上”
- `JVM flags` 解决的是“JVM 如何选择 GC、线程数、暂停目标、兼容性参数等”
- `-XX:ActiveProcessorCount=N` 位于两者之间，它不是亲和性，但可以让 JVM 按 `N` 个 CPU 做启发式决策

## 结论先行

建议将需求拆成三层，而不是只做一个“CPU 数量”字段：

1. `CPU affinity / CPU set`
   控制进程或容器能使用哪些逻辑 CPU。
2. `Active processor count`
   通过 `-XX:ActiveProcessorCount=N` 告诉 JVM 按多少 CPU 工作。
3. `JVM flags preset`
   给用户可选的 JVM / GC / Minecraft 常见优化参数组合。

推荐总体方向：

- 本地运行时优先支持 `jar` / `starter` 直启模式的 affinity
- Docker 运行时优先支持 `cpuset-cpus`，必要时再补 `--cpus`
- 当用户设置了 CPU 核心数上限时，默认同时追加 `-XX:ActiveProcessorCount=N`
- 预设 flags 不直接存成一整段字符串，而是存“预设 ID + 用户附加参数”

## 目标

- 允许用户为单个服务端配置 CPU 相关限制
- 允许用户选择一个或多个 JVM flags 预设策略
- 对本地与 Docker 运行时给出一致的用户心智模型
- 不破坏现有 `jvm_args` 字段与已有服务器记录
- 为后续高级模式预留扩展位

## 非目标

- 首期不承诺自动寻找“最佳核心”
- 首期不实现 NUMA 绑定
- 首期不实现按不同 MC 核心动态调参的自动调优器
- 首期不保证 `bat/sh/ps1/custom` 脚本模式的 affinity 100% 命中最终 Java 子进程
- 首期不做复杂的“多预设叠加冲突解析器”

## 术语约定

### CPU 亲和性

将进程或容器绑定到特定逻辑 CPU 集合，例如 `0-3` 或 `1,3,5`。

### CPU 核心数量限制

用户只指定数量 `N`，不指定具体核心编号。系统需要把数量映射成一个具体 CPU 集合。

### Active Processor Count

JVM 参数 `-XX:ActiveProcessorCount=N`，用于覆盖 JVM 自动检测到的 CPU 数量。

### Flags 预设

命名后的 JVM 参数集合，例如：

- `none`
- `aikar-g1`
- `g1-basic`
- `throughput-basic`

## 用户侧配置模型建议

建议不要只增加一个 `cpu_cores` 字段，而是改为显式表达“模式”。

### 面向用户的概念模型

- `CPU 限制模式`
  - `off`：不限制
  - `count`：只指定数量 `N`
  - `explicit`：直接指定核心集合字符串，如 `0-3,6,7`
- `是否同步限制 JVM 可见 CPU 数`
  - 默认开启
- `JVM flags 预设`
  - 默认 `none`
- `用户附加 JVM 参数`
  - 保留现有自由输入能力

### 建议的数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuPolicyConfig {
    pub mode: CpuPolicyMode,
    pub count: Option<u16>,
    pub explicit_set: Option<String>,
    pub sync_active_processor_count: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CpuPolicyMode {
    #[default]
    Off,
    Count,
    Explicit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JvmPresetConfig {
    pub preset_id: String,
    pub extra_args: Vec<String>,
}
```

### 在现有模型中的放置建议

建议分别放到本地与 Docker 运行时配置里，而不是提升到 `ServerInstance` 顶层统一字段。

原因：

- 本地与 Docker 的落地方式不同
- Docker 未来可能既支持 `cpuset-cpus`，也支持 `cpus` 配额，两者不完全等价
- 本地脚本模式存在可支持性差异，运行时内部更容易表达能力边界

示意：

```rust
pub struct LocalRuntimeConfig {
    pub jar_path: String,
    pub startup_mode: String,
    pub custom_command: Option<String>,
    pub java_path: String,
    pub jvm_args: Vec<String>,
    pub cpu_policy: Option<CpuPolicyConfig>,
    pub jvm_preset: Option<JvmPresetConfig>,
}

pub struct DockerItzgRuntimeConfig {
    pub image: String,
    pub image_tag: String,
    pub container_name: String,
    pub type_value: String,
    pub version: String,
    pub data_dir_mount: String,
    pub published_game_port: u16,
    pub env: BTreeMap<String, String>,
    pub extra_ports: Vec<PublishedPort>,
    pub volume_mounts: Vec<VolumeMount>,
    pub docker_backend_kind: DockerBackendKind,
    pub command_mode: DockerCommandMode,
    pub rcon: Option<RconConfig>,
    pub cpu_policy: Option<CpuPolicyConfig>,
    pub jvm_preset: Option<JvmPresetConfig>,
}
```

## 兼容性策略

### 现有 `jvm_args` 的处理

现有 `jvm_args` 不应废弃，建议定义为“最终用户自定义参数层”。

最终合成顺序建议为：

1. Sea Lantern 内部保底参数
2. 预设 flags
3. CPU 相关自动注入参数，例如 `-XX:ActiveProcessorCount=N`
4. 用户 `jvm_args`

这样做的好处：

- 兼容已有服务器配置
- 保留用户覆盖能力
- 预设逻辑可独立演进

### 冲突规则建议

若用户手动写入以下参数，系统不应重复注入：

- `-XX:ActiveProcessorCount=`
- 与预设完全同名的 JVM 参数

如果检测到冲突，建议：

- 启动时记录一条 Sea Lantern 日志
- 使用“用户参数优先，自动注入跳过”的策略

## 运行时映射建议

## 本地运行时

### Windows

可选实现路线：

1. 启动前包装 `cmd /c start /b /affinity ...`
2. `spawn` 后按 PID 调用系统 API 设置 affinity

建议优先级：

- `jar` / `starter` 直启模式：优先考虑 `spawn` 后按 PID 设置 affinity
- `bat/sh/ps1/custom`：首期不承诺完全支持，最多标记为 best-effort

原因：

- `start /affinity` 会引入额外壳层与参数转义复杂度
- 当前本地直启已经集中在命令构建与 `spawn` 点，后置按 PID 设置更容易与现有代码对接
- 脚本模式可能由脚本再次拉起 Java，导致外层 shell 与最终 Java 进程不是同一个 PID

### Linux

可选实现路线：

1. 启动前用 `taskset -c <set> java ...`
2. `spawn` 后用 `taskset -pc <set> <pid>`
3. 直接调用 `sched_setaffinity`

建议优先级：

- 直启模式优先考虑系统调用或 `spawn` 后设置
- 脚本模式同样保守处理

### 本地运行时语义矩阵

| 启动模式  | 是否建议首期支持 affinity | 说明                                   |
| --------- | ------------------------- | -------------------------------------- |
| `jar`     | 是                        | Sea Lantern 直接构造 Java 命令，最稳定 |
| `starter` | 是                        | 本质仍接近直启                         |
| `bat`     | 谨慎                      | 可能只绑定到外层脚本进程               |
| `sh`      | 谨慎                      | 同上                                   |
| `ps1`     | 谨慎                      | 同上                                   |
| `custom`  | 谨慎                      | 不能假设最终进程模型                   |

首期建议对外明确：

- `jar` / `starter` 为正式支持
- 其余模式暂不支持或仅 best-effort

## Docker 运行时

Docker 运行时应优先采用官方原生资源限制语义。

### 推荐映射

- `cpu_policy.mode = explicit`
  - 映射到 `--cpuset-cpus=<set>`
- `cpu_policy.mode = count`
  - 转换成 `0-(N-1)` 后映射到 `--cpuset-cpus`
- 未来可选补充
  - `--cpus=<float>` 作为“配额模式”而非“亲和性模式”

### 不建议首期混淆的两种能力

- `--cpuset-cpus`
  限定“能在哪些核心上跑”
- `--cpus`
  限定“总共最多可吃多少 CPU 时间份额”

这两个概念对用户感知不同，不建议共用一个字段。

## JVM 参数合成建议

### `ActiveProcessorCount`

当以下条件满足时，建议自动注入：

- `cpu_policy.mode != off`
- `sync_active_processor_count = true`
- 用户未手动提供 `-XX:ActiveProcessorCount=`

注入值规则：

- `count` 模式：直接取 `N`
- `explicit` 模式：取 CPU 集合解析后的元素个数

### 为什么建议默认开启

如果只做 affinity，不做 `ActiveProcessorCount`，JVM 仍可能按宿主机全部 CPU 数做线程启发式。

这会导致：

- GC 线程数估计偏大
- ForkJoinPool 等并发组件估计偏大
- 用户设置了“限制核心数”，但 JVM 行为看起来仍像没限制

## JVM Flags 预设建议

首期建议只做“单选预设 + 用户附加参数”，不要做多选叠加。

### 推荐预设 ID

- `none`
- `g1-basic`
- `aikar-g1`
- `throughput-basic`
- `paper-recommended-lite`

### 预设元数据建议

```rust
pub struct JvmPresetDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub args: &'static [&'static str],
    pub min_java_version: Option<u8>,
    pub max_java_version: Option<u8>,
    pub recommended_heap_min_mb: Option<u32>,
    pub recommended_heap_max_mb: Option<u32>,
    pub notes: &'static [&'static str],
}
```

### 为什么不建议只做一个 Aikar 预设

原因：

- Aikar flags 主要针对特定时代的 G1 调优经验
- Java 8 / 17 / 21 的参数适配并不完全相同
- 小内存服与大内存服适用性不同
- 不同核心类型的默认建议不同

因此预设至少应带上：

- Java 版本适用范围
- 堆内存适用范围
- 简短说明与风险提示

## UI 与交互建议

### 首期表单建议

- `CPU 限制`
  - 开关：关闭 / 按核心数量 / 按核心列表
- `核心数量`
  - 整数输入，要求 `N > 0`
- `核心列表`
  - 字符串输入，如 `0-3,6,7`
- `同步限制 JVM 可见 CPU 数`
  - 默认开启
- `JVM 预设`
  - 下拉选择
- `额外 JVM 参数`
  - 保留现有输入能力

### 文案建议

必须显式告诉用户：

- Minecraft 主线程并不会因为绑定更多核心而线性提速
- CPU affinity 更适合“隔离资源”和“减少抢占”
- `ActiveProcessorCount` 是 JVM 线程启发式设置，不是操作系统级绑定

## 校验规则建议

### 数量模式

- `N > 0`
- `N <= host logical cpu count`
- 若当前无法获取主机 CPU 数，可只校验 `N > 0`，启动时再做最终校验

### 显式核心列表模式

- 允许格式：`0-3`, `1,3,5`, `0-3,6,7`
- 不允许负数
- 不允许空片段
- 不允许重复区间解析后溢出
- 最终解析出的核心数量必须 `> 0`

### 预设选择

- 预设若与当前 Java 版本不兼容，应给出阻止或强警告
- 预设若与当前堆大小不匹配，应至少给出提示

## 启动日志建议

每次启动时建议把最终决策写入 Sea Lantern 日志，便于排障。

例如：

```text
[Sea Lantern] CPU policy applied: mode=count count=4 cpuset=0-3 sync_active_processor_count=true
[Sea Lantern] JVM preset applied: aikar-g1
[Sea Lantern] ActiveProcessorCount injected: 4
```

若跳过注入，也应记录原因：

```text
[Sea Lantern] ActiveProcessorCount skipped: user already provided explicit value
```

## 实现接入点建议

基于当前仓库结构，预计会落在以下位置：

- 本地直启命令构造
  - `src-tauri/src/services/server/manager/runtime_start/launch/command_builder.rs`
- 本地 `spawn` 点
  - `src-tauri/src/services/server/manager/runtime_start/launch/runner.rs`
- 服务器运行时数据模型
  - `src-tauri/src/models/server.rs`
- Docker CLI 运行时参数生成
  - `src-tauri/src/utils/cli/server_docker.rs`
- Docker runtime 真实启动链路
  - `src-tauri/src/services/server/runtime/docker_itzg.rs`

## 分阶段实现建议

### Phase 1

只做 JVM flags 预设，不做 affinity。

收益：

- 风险最低
- 与现有 `jvm_args` 最兼容
- 可以先打通 UI、模型、合成与日志链路

### Phase 2

做 Docker `cpuset-cpus` + `ActiveProcessorCount`。

收益：

- 语义最清晰
- 官方能力稳定
- 容器运行时比脚本模式更容易控制

### Phase 3

做本地 `jar` / `starter` 模式 affinity。

收益：

- 覆盖最稳定的本地主路径
- 避免一开始就掉进脚本模式复杂性

### Phase 4

评估是否支持 `bat/sh/ps1/custom` 的 best-effort affinity。

此阶段需要先回答：

- 是只绑定外层进程，还是继续跟踪最终 Java 进程
- 若脚本多次 fork，Sea Lantern 是否有能力识别最终托管对象

## 风险与注意事项

### 风险 1：用户把“核心数限制”理解成“性能提升开关”

需要通过 UI 文案与文档明确预期。

### 风险 2：脚本模式实现后用户误以为 100% 生效

必须在产品语义上区分“正式支持”与“尽力支持”。

### 风险 3：预设 flags 过时

预设定义必须集中维护，避免散落在 UI 文案或多处硬编码。

### 风险 4：自动注入与用户自定义参数冲突

需要明确“用户参数优先”规则，并记录启动日志。

## 推荐 MVP

如果只选一个最稳妥的最小可用集合，建议是：

1. 新增 `jvm_preset` 单选配置
2. 保留现有 `jvm_args`
3. Docker 运行时新增 `cpu_policy`，先只支持 `count` 和 `explicit`
4. 本地运行时先只在 `jar` / `starter` 模式支持 `count`
5. 默认开启 `sync_active_processor_count`

这样可以尽快覆盖主要需求，同时把高风险部分留到后续阶段。

## 待确认问题

- 本地 Windows affinity 最终采用 `spawn` 后设 PID，还是命令包装方式
- 本地 Linux affinity 是直接系统调用，还是先用外部命令实现
- 预设列表首发是否只放 2 到 3 个，避免维护面过大
- `count` 模式是否固定映射到前 `N` 个逻辑 CPU，还是预留“自动挑选低编号核心”之外的策略
- Docker CLI 当前生成链路是否已经具备添加 `--cpuset-cpus` 的统一参数入口，还是需要先补抽象层

## 建议的下一步

在真正实现前，建议先完成以下两件事：

1. 确认最终配置 schema 与兼容迁移策略
2. 画出本地与 Docker 启动链路中的参数注入顺序

完成后再进入编码，会比直接改启动逻辑更稳。