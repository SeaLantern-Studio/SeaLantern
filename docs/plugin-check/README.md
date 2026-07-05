# SeaLantern 插件 Trusted 审查与校验流程

## 目的

本文档用于总结 SeaLantern 当前的 **Trusted 插件审查与校验流程**。

重点说明：

- 当前 Trusted 插件是如何被开发组“审查并放行”的；
- 客户端安装/启用时如何判断一个插件是否属于 Trusted；
- 当前流程里哪些能力已经实现，哪些还只是目标方向。

本文档只描述 **当前代码已经落地的流程**，不把未来的 `.sig` / 公钥验签链写成现状。

---

## 一句话总结

当前 SeaLantern 的 Trusted 审查机制本质上是：

> **开发组人工审查 + 内置 trusted catalog + 包 hash 命中校验 + 权限上限校验 + 启用时 hash 授权**

它**还不是**完整的“签名文件分发 + 公钥验签”体系。

---

## 当前流程边界

### 当前已经有的能力

1. Trusted 插件专用 catalog
2. 插件包 `archive_sha256` 校验
3. manifest 权限与 `permission_ceiling` 对比
4. `revoked` 撤销标记
5. 启用时按 `hash` 级别确认授权
6. 安装后内容被修改时自动降级

### 当前还没有的能力

1. `trusted-catalog.sig`
2. 内置 `trust_root_public_key`
3. catalog 数字签名验签
4. 在线 signed catalog 更新
5. 在线 `isSigned(plugin_id, version, hash)` 查询

因此，当前如果对外描述该机制，应该称为：

- **Trusted 审查与 hash 校验流程**

而不是：

- **完整签名信任链**

---

## 当前 Trusted 审查流程

## 第 1 步：开发组人工审查插件

当前 Trusted 插件不会自动产生，也不是插件作者自报 `trusted` 就能进入。

进入 Trusted 路径前，开发组需要人工审查至少这些内容：

1. 插件源码或可审查仓库
2. 发布包内容
3. 插件声明的高权限能力
4. 允许的权限上限
5. 发布者身份与仓库来源

如果审查通过，才进入下一步 catalog 录入。

---

## 第 2 步：把审查结果写入 trusted catalog

当前审查结果的落地点是：

- `shared/plugin-trusted-catalog.json`

每个 Trusted 条目当前包含：

- `plugin_id`
- `version`
- `archive_sha256`
- `publisher_id`
- `review_class`
- `permission_profile`
- `permission_ceiling`
- `repository`
- `license`
- `reviewed_at`
- `revoked`
- `notes`

其中最关键的是三类事实：

1. **这个 Trusted 身份对应哪个具体包**：`plugin_id + version + archive_sha256`
2. **它最多允许申请哪些权限**：`permission_ceiling`
3. **它是否已被撤销**：`revoked`

---

## 第 3 步：随应用打包内嵌 catalog

当前 trusted catalog 不是运行时远程获取的，也不是单独签名文件。

当前后端通过：

- `include_str!("../../../../shared/plugin-trusted-catalog.json")`

把 catalog 作为 **bundled snapshot** 直接编译进应用。

这意味着当前审查结果的发布方式本质上是：

1. 更新仓库中的 `shared/plugin-trusted-catalog.json`
2. 构建并发布新的 SeaLantern 应用版本

也就是说：

> 当前 Trusted 审查结果与应用版本绑定。

---

## 第 4 步：安装时判断插件是否可进入 Trusted 路径

安装时并不是所有插件都会进入 Trusted 评估。

当前逻辑是：

### 普通插件路径

如果插件没有申请 Trusted-only 权限，则不会进入 Trusted 审查链。

它会按：

- `standard_sandbox`
- 或 `unreviewed`

进行分流。

### Trusted 候选路径

如果插件申请了 Trusted-only 权限，例如：

- `execute_program`
- `process.exec`
- `process.inspect`
- `process.output.read`
- `process.kill`
- `plugin_folder_access`
- `plugins.read`
- `plugins.write`
- `ui.component.proxy`

才会进入 Trusted 评估链。

---

## 第 5 步：安装时执行 Trusted 校验

当前 Trusted 校验发生在后端 `assess_plugin(...)` 中。

### 当前校验顺序

1. 判断插件是否请求 Trusted-only 能力
2. 检查安装来源是否提供 `archive_sha256`
3. 从内嵌 catalog 中查找 `(plugin_id, version)`
4. 检查条目是否 `revoked`
5. 检查 `archive_sha256` 是否完全匹配
6. 检查 manifest 权限是否全部落在 `permission_ceiling` 内

### 全部通过后的结果

插件会被标记为：

- `trust_level_display = trusted`
- `execution_class = trusted_full`
- `review_status = sealantern_reviewed`
- `integrity_status = verified_hash`
- `trusted_policy_source = bundled_snapshot`
- `hash_matched = true`

注意这里当前是：

- `verified_hash`

不是：

- `verified_signature`

因为当前还没有 `.sig` 验签链。

---

## 第 6 步：校验失败时如何处理

只要以下任一条件失败：

- 找不到 catalog 条目
- 条目已撤销
- `archive_sha256` 不匹配
- manifest 权限超出 `permission_ceiling`

插件都**不会**进入 Trusted。

当前会降回：

- `unreviewed`

而不是回退成普通第三等标准沙箱插件。

这条规则的目的很明确：

> 声称自己要高权限能力，但又没有命中 Trusted 审查结果的插件，不能被误当成普通插件处理。

---

## 第 7 步：启用时再次确认 Trusted 授权

即使安装时已经识别为 Trusted，启用时也不是自动放行。

当前启用确认规则是：

- `builtin`：不需要授权
- `trusted`：需要 `hash` 级授权
- `standard_sandbox`：只有 `requires_explicit_consent = true` 才需要 `version` 级授权
- `unreviewed`：需要 `version` 级授权

其中 Trusted 的关键点是：

> 用户确认的是“当前这个版本 + 当前这个 hash”，不是只按插件名放行。

这使得 Trusted 授权不会因为插件作者替换包内容而被直接继承。

---

## 第 8 步：安装后内容变化时自动降级

当前实现还有一条运行时完整性保护：

1. 安装完成后会记录 `installed_tree_sha256`
2. 重新扫描插件目录时会重新计算插件目录树 hash
3. 如果目录内容与安装时记录不一致，则自动降级

降级后通常会变成：

- `trust_level_display = unreviewed`
- `integrity_status = mismatch`
- `hash_matched = false`
- `verified_hash = None`
- `requires_explicit_consent = true`

也就是说：

> 当前 Trusted 身份不是“一次判定永久有效”，插件内容变了就会掉级。

---

## 当前流程中的角色分工

### 开发组负责

1. 审查插件源码与发布包
2. 决定是否允许进入 Trusted
3. 决定 `permission_ceiling`
4. 更新 `shared/plugin-trusted-catalog.json`
5. 在必要时设置 `revoked = true`

### 客户端负责

1. 本地读取内嵌 catalog snapshot
2. 本地执行 hash 与权限上限比对
3. 本地判断 Trusted / Unreviewed
4. 启用时要求用户对当前 hash 再次授权
5. 插件内容变化时自动降级

### 用户负责

1. 决定是否启用 Trusted 插件
2. 对高权限插件带来的风险自行承担最终责任
3. 在插件升级、hash 变化或撤销后重新做启用确认

---

## 当前流程的优点

1. 不依赖在线 `isSigned` 服务
2. 离线可判定
3. 对普通插件生态没有额外签名负担
4. 对 Trusted 插件已经有真实的 hash 绑定与掉级保护
5. 不会因为“仓库看起来可信”就跳过当前包校验

---

## 当前流程的限制

1. catalog 更新需要随应用版本分发
2. 还没有独立的签名证明链
3. 还没有 catalog 验签失败回退到旧签名 snapshot 的完整逻辑
4. 还不适合更大规模的 Trusted 第三方生态

因此，当前流程更适合：

- 少量高权限、开发组明确审查过的 Trusted 插件

而不是：

- 大规模开放签名生态

---

## 当前不应误称的内容

为了避免对外说明失真，当前不应把该流程表述成：

- “插件已经完成数字签名验证”
- “当前已有完整签名审查链”
- “当前已有远程签名同步机制”

更准确的表述应是：

- “当前 Trusted 插件采用开发组审查 + 内置 trusted catalog + hash 校验机制”

---

## 未来目标态

未来如果要把这条链补全，目标通常会是：

1. 保留 `trusted-catalog.json`
2. 增加 `trusted-catalog.sig`
3. 应用内置 `trust_root_public_key`
4. 客户端本地验签 catalog
5. 可选支持在线更新 signed catalog

但这部分目前仍是**目标态**，不是当前实现。

---

## 相关代码位置

### 当前 Trusted 校验核心

- `backend/tauri-host/src/services/plugin_trusted_catalog.rs`

### 当前 Trusted catalog 数据

- `shared/plugin-trusted-catalog.json`

### 市场安装时计算 archive hash

- `backend/tauri-host/src/commands/plugins/manage/market/install.rs`

### 扫描时做目录树完整性降级

- `backend/tauri-host/src/plugins/manager/source_local.rs`

### 插件信任相关字段模型

- `backend/tauri-host/src/models/plugin_model/runtime.rs`
- `frontend/src/types/plugin.ts`

---

## 当前结论

当前 SeaLantern 的 Trusted 机制可以准确定义为：

> **人工审查 + 内置 trusted catalog + hash 命中校验 + 权限上限校验 + 启用时 hash 授权 + 安装后改动自动降级**

它已经足够支持“小规模高权限 Trusted 插件”模型，
但还不能称为“完整签名信任链”。
