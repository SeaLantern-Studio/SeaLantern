# Agent5 校准结论（2026-06-02）

## 结论

- 当前工作区源码层面不存在“`src-tauri/src/adapters/http/server.rs` 缺失导致无法编译”这个问题。
- 当前树可以通过 `cargo check`，因此“仓库原本就坏，导致本轮无法验证”不能作为统一解释继续沿用。
- Agent 1 基本完成，但还留有 token 日志泄露和单文件大小上限两个尾巴。
- Agent 2 的实现方向基本对，但验证结论不可信，主要问题在测试过滤方式和失败归因。
- Agent 3 的结构拆分基本成立，但仍需补一轮行为验证。
- Agent 4 只完成了 `local_helper` 这一半，没有推进 `manager.rs`，本轮完成度不足。

## Review 判断

### Agent 1

- 状态：基本完成，但存在新增异常风险
- 已完成：默认 loopback 绑定、Bearer 鉴权、显式 CORS 白名单、上传流式写盘、文件名净化、目录边界校验、文件数限制、UUID 去重
- 未收口：
  - 未配置静态 token 时会把完整 token 打进日志
  - 缺少单文件大小上限语义

### Agent 2

- 状态：方向对，但验证不成立
- 已完成：前台 exec 真超时、权限统一定义、`plugins -> plugin_folder_access` 归一化
- 问题：把“0 tests / 过滤串未命中”误归因为全局编译失败

### Agent 3

- 状态：基本完成
- 已完成：`docker_support.rs` 拆成 `docker_preflight`、`docker_request_builder`、`docker_mounts`、`docker_paths` 四层，外部入口仍由 facade 承接
- 未收口：CLI/Docker 相关行为验证还不够实

### Agent 4

- 状态：完成一半
- 已完成：`local_helper` 拆出 `state.rs`、`protocol.rs`
- 未完成：`manager.rs` 完全未推进，因此“服务器管理复杂度下降”还不能成立

## 下一轮顺序

### 第一批：并行执行

1. Agent 1 跟进
   - 只收尾 HTTP 剩余风险
   - 重点：token 日志泄露、单文件大小上限、补精确测试

2. Agent 2 跟进
   - 不继续扩功能
   - 只做真实验证，修正错误测试过滤，必要时做最小修补

### 第二批：顺延执行

3. Agent 3 跟进
   - 只补 CLI Docker 行为验证
   - 暂不继续扩大结构拆分

### 暂缓

4. Agent 4 暂停
   - 下一轮若重开，必须明确要求“要么触达 `manager.rs`，要么直接判定本轮不完整”

## 分工颗粒度结论

- 插件运行时和 `local_helper` 的边界最清楚
- HTTP 安全面和上传链路不应该拆给不同 Agent，它们共享 `src-tauri/src/adapters/http/server.rs`
- `Agent 4` 这类“复杂度拆分”任务，必须明确是否要求触达 `manager.rs`，否则完成度会持续模糊
