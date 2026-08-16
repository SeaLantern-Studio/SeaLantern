# 来源与本地调整

本目录基于 `mihomo-party-org/sysproxy-rs-opti`：

- 上游仓库：<https://github.com/mihomo-party-org/sysproxy-rs-opti>
- 基准提交：`0e242650d7b4c568ab1b91cb319e0b44b2a445bd`
- 上游版本：`0.5.1`
- 许可证：MIT，见 `LICENSE`

为缩小本项目的供应链与编译面，本地副本移除了未使用的 N-API 绑定、代理守护器、
工具函数、基准测试和 JavaScript 包装，只保留 Windows、Linux 与 macOS 的核心系统
代理实现。同时将上游 `url` 的 `<2.5` 版本上限放宽为工作区已使用的兼容 `2.x`。

本地还收紧了只读解析：Windows 多协议配置不再把缺少 `http=` 的 SOCKS 等条目回退
为 HTTP 代理；Linux KDE 配置必须使用与目标服务一致的 scheme，且不得携带用户信息、
路径、查询或片段。除此之外，保留的平台实现来自上述基准提交；rustfmt 产生的差异仅
为机械排版。
