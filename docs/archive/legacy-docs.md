# 旧文档摘要

本页只保留从旧文档迁移出的必要上下文。原始全文不再作为仓库文档维护。

| 原文                      | 处理                                                                           | 当前依据                                              |
| ------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------- |
| `docs/architecture.md`    | 删除旧的单文件版本和其中的临时迁移章节；保留当前分层、宿主和调用边界的必要内容 | [`design/architecture.md`](../design/architecture.md) |
| `docs/CONTRIBUTING.md`    | 删除包含旧工具版本、旧命令和过时错误处理建议的全文                             | [`rules/contributing.md`](../rules/contributing.md)   |
| `docs/language-system.md` | 删除错误的 `language/` 路径和过时 API 清单；保留 i18n 的当前契约摘要           | [`design/frontend.md`](../design/frontend.md)         |
| `docs/theme-system.md`    | 删除错误的 `themes/` 路径和完整主题样例；保留四种当前 `ColorPlan` 的说明       | [`design/frontend.md`](../design/frontend.md)         |
| `docs/ui库.md`            | 删除易随依赖版本漂移的完整组件属性快照；保留 UI 依赖和入口约束                 | [`design/frontend.md`](../design/frontend.md)         |
| `docs/lua-api/*`          | 按当前插件 API v2 重构需要全部删除；旧函数路径和旧权限语义不再作为参考         | [`design/plugin.md`](../design/plugin.md)             |

## 不应从归档恢复的内容

- 已删除的 Docker 宿主和发布流程。
- 旧 Lua API、隐式权限字段和旧插件路径。
- 以迁移完成为前提、但当前源码仍未完成的目标状态。
- 只对某个旧 PR 或某次本地排查有意义的临时过程。
