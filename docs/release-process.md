# 发布流程

这份文档描述当前 Sea Lantern 仓库的版本发布方式，内容以仓库内现有脚本和 GitHub Actions workflow 为准。

## 概览

- 正式版本：手动更新语义化版本号，推送正式 tag，GitHub Actions 自动构建多平台桌面产物并创建 Draft Release。
- Nightly 版本：从 `beta` 分支定时或手动触发，不改清单语义化版本号，通过构建环境注入显示版本并发布为 prerelease。
- Release 发布后续：正式 Release 被手动发布后，会继续触发 AUR、CNB、Docker 镜像等后续流程。

## 一、正式版本发布

### 1. 更新版本号

仓库提供了版本辅助脚本：

```bash
pnpm sv
pnpm cv 1.2.3
```

- `pnpm sv`：查看当前版本状态。
- `pnpm cv <version>`：写入新的语义化版本号。

当前脚本会同步更新这些文件：

- `frontend/package.json`
- `backend/tauri-host/Cargo.toml`
- `backend/tauri-host/tauri.conf.json`

如果仓库根目录下存在 `PKGBUILD` 或 `.SRCINFO`，脚本也会一并尝试同步。

### 2. 提交并推送到 `main`

建议顺序：

```bash
pnpm cv 1.2.3
git add frontend/package.json backend/tauri-host/Cargo.toml backend/tauri-host/tauri.conf.json Cargo.lock
git commit -m "chore(release): bump version to 1.2.3"
git push origin main
```

正式发布 workflow 会校验：

- tag 指向的提交是否位于 `main` 分支上；
- 该提交是否已经通过 `代码检查` workflow。

因此不要在检查未通过时直接打正式 tag。

### 3. 推送正式 tag

当前正式发布 workflow 支持两种 tag 前缀：

- `v*`
- `sea-lantern-v*`

常见用法：

```bash
git tag v1.2.3
git push origin v1.2.3
```

或：

```bash
git tag sea-lantern-v1.2.3
git push origin sea-lantern-v1.2.3
```

Tag 必须能解析出语义化版本号，例如：

- `v1.2.3`
- `sea-lantern-v1.2.3`
- `v1.2.3-rc.1`

### 4. GitHub Actions 自动构建 Draft Release

推送 tag 后，`正式发布` workflow 会自动：

- 构建 Windows / macOS / Linux 桌面产物；
- 上传安装包和便携包；
- 创建 GitHub Release；
- 当前配置下会创建为 **Draft Release**，不是自动 published。

因此正式发版仍有一个人工步骤：

1. 等 `正式发布` workflow 完成；
2. 在 GitHub Release 页面检查产物和说明；
3. 手动点击 `Publish release`。

## 二、正式 Release 发布后的自动流程

当 Draft Release 被手动发布为 `published` 后，后续 workflow 会继续执行。

### 1. AUR

`同步到AUR` workflow 会：

- 提取 release tag 中的版本号；
- 更新 `packaging/PKGBUILD`；
- 生成 `.SRCINFO`；
- 推送到 AUR。

### 2. CNB

`同步到CNB` workflow 会在 release 发布后自动同步，也支持手动指定 tag 补同步。

### 3. Docker 镜像

`Docker镜像发布` workflow 监听 `正式发布` 和 `夜间发布` workflow 成功结束事件，再继续构建和推送镜像。

当前会发布：

- 正式版本 tag
- `latest`
- Nightly 的 `nightly`

## 三、Nightly 版本发布

Nightly 与正式版不同：

- 来源分支：`beta`
- 触发方式：定时任务或手动触发 `夜间发布`
- 不直接把 `NightlyBuild-*` 写回版本清单
- 通过环境变量 `SEA_LANTERN_BUILD_VERSION` 注入显示版本

当前 Nightly 显示版本格式：

```text
NightlyBuild-<SHORT_SHA>-<UTC_DATETIME>
```

当前 Nightly tag 格式：

```text
nightly-v<SEMVER>-<YYYYMMDD>-<SHORT_SHA>
```

例如：

```text
NightlyBuild-abcdef0-20260705T120000Z
nightly-v1.2.3-20260705-abcdef0
```

Nightly workflow 会：

- 基于 `beta` 分支当前提交创建或复用 prerelease；
- 构建多平台桌面产物；
- 上传便携包等资产；
- 清理旧的 Nightly Release。

## 四、客户端更新来源

当前更新检查逻辑大致如下：

- 非 Linux：直接走 GitHub Release 更新检查；
- Linux：
  - Arch Linux：优先 AUR；
  - 其他 Linux：走 CNB + GitHub 的组合检查。

因此对正式版本而言，GitHub Release 仍然是最核心的发布源。

## 五、推荐的正式发版操作顺序

```bash
pnpm cv 1.2.3
git add frontend/package.json backend/tauri-host/Cargo.toml backend/tauri-host/tauri.conf.json Cargo.lock
git commit -m "chore(release): bump version to 1.2.3"
git push origin main
```

等待 `代码检查` 通过后：

```bash
git tag v1.2.3
git push origin v1.2.3
```

然后：

1. 等 `正式发布` workflow 构建完成；
2. 检查 GitHub 上生成的 Draft Release；
3. 手动点击发布；
4. 让 AUR / CNB / Docker 后续链路继续自动执行。

## 六、注意事项

### 1. 不建议用本地手工构建结果直接当正式 Release 产物

仓库当前正式发布路径以 GitHub Actions 产物为准，本地构建更适合开发验证，不适合作为正式分发来源。

### 2. `pnpm cv` 只写语义化版本

`pnpm cv` 目前不负责写入 NightlyBuild 形式的版本号。

NightlyBuild 显示版本应通过构建环境变量注入，例如：

```text
SEA_LANTERN_BUILD_VERSION=NightlyBuild-abcdef0-20260705T120000Z
```

### 3. 正式发布前先确认 CI 绿灯

虽然 workflow 自己也会校验，但从维护流程上仍建议先确认：

- `代码检查` 已通过；
- 需要的 release 资产说明无误；
- tag 版本号与清单文件一致。

