# Roadmap

这里记录 Sea Lantern 的当前产品边界和工程优先级，不替代 Issue，也不承诺发布日期。路线图只保留仍然有决策价值的事项；已经存在的代码能力不重复写成“待开发”。

## 当前基线

- Desktop 宿主是 `src-tauri`，Web 宿主是 `server`；二者共享 `application` 服务装配和业务实现。
- Rust workspace 当前包含 `application`、`core`、`contract`、`feature`、`infra`、`server`、`src-tauri` 以及两个独立许可的 vendored crate。
- 前端是根目录 `src/` 下的 Vue 3 + TypeScript + Vite 应用。
- 插件当前按 API v2 实现，包含 Lua runtime、能力声明、授权策略、会话审批和审计。
- Docker 集成已经从当前 workspace 和发布流程移除，不属于当前产品承诺。

## 工程优先级

| 优先级 | 方向                       | 完成判断                                                                                           |
| ------ | -------------------------- | -------------------------------------------------------------------------------------------------- |
| P0     | 文档公开化与持续同步       | 贡献者仅通过仓库内 `docs/` 即可找到当前架构、规则和路线图；源码结构变更有对应文档变更              |
| P1     | Desktop/Web API 覆盖收敛   | `src/api` 中需要跨宿主的业务都有明确的 Tauri 与 Axum 契约；未支持能力保持显式不可用                |
| P1     | RESTful RPC 迁移收尾       | 普通业务统一由单一 Axum router 提供资源/动作 REST；旧 generic RPC 清理，插件 RPC 保持显式例外      |
| P1     | 契约层边界收敛             | `crates/contract` 只拥有共享 DTO/错误；服务端口归 `application::port`，功能实现归 `crates/feature` |
| P1     | 插件 v2 宿主边界完善       | 新能力都经过 manifest、scope、trust、授权和审计；Desktop/Web 不复制策略实现                        |
| P2     | 现有业务的可靠性与可观测性 | 服务器生命周期、下载、备份、定时任务、更新和在线隧道的失败路径有针对性测试与可诊断日志             |

## 暂不纳入当前路线

- 不恢复已删除的 Docker 宿主、旧兼容命令或旧 Lua API 文档，除非先提交独立设计并说明新的实际消费者。
- 不为普通业务引入覆盖所有宿主的通用 RPC runtime；插件 RPC 是显式例外。
- 不把一次性排查、未批准方案或具体 PR 交接写入长期路线图；这些内容放在 `docs/tmp/`，确认后再转移。

## 更新方式

每次完成一个路线图事项时，应同时更新实现对应的设计/规则文档，并删除已经失效的临时记录。路线图条目若连续多个版本没有实际消费者或验收标准，应移入 `archive` 或删除，而不是继续累积。
