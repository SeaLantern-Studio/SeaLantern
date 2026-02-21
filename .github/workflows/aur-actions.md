## 📦 添加 AUR 自动发布工作流

### 主要改动

添加 GitHub Actions 工作流，实现 SeaLantern 的 AUR 包自动发布。

### ✨ 功能特性

- **自动触发**：当发布新 Release 时自动更新 AUR 包
- **手动触发**：支持手动指定版本发布
- **版本管理**：自动更新 PKGBUILD 中的版本号
- **文件生成**：自动生成 `.SRCINFO` 文件
- **安全配置**：使用 SSH 密钥安全认证

### 🛠️ 工作流程

1. **检出代码** - 获取最新源码
2. **安装依赖** - 安装必要的构建工具
3. **配置 SSH** - 设置 AUR 的 SSH 认证
4. **更新版本** - 将 PKGBUILD 中的版本号更新为目标版本
5. **生成文件** - 创建 `.SRCINFO` 文件
6. **发布** - 推送到 AUR 仓库

### 🔧 使用方法

#### 自动发布

- 创建 GitHub Release 时自动触发
- 版本号从 Release tag 获取

#### 手动发布

1. 进入 Actions → "Publish to AUR"
2. 点击 "Run workflow"
3. 输入版本号（如 `0.7.3`）
4. 可选：勾选 "Skip makepkg test" 跳过测试

### 🔐 需要设置的 Secrets

在 GitHub 仓库设置中添加：

- `AUR_SSH_PRIVATE_KEY` - AUR 的 SSH 私钥
- 将公钥添加到 AUR 账户的 SSH keys 中（发送给AUR仓库维护者***xuezhaju***[见github]）

### ✅ 测试情况

- [x] 版本号正确更新
- [x] `.SRCINFO` 正确生成
- [x] SSH 认证正常
- [x] 手动触发工作正常

### 🔗 相关链接

- AUR 包页面：https://aur.archlinux.org/packages/sealantern
- GitHub Actions 文档：https://docs.github.com/actions
- 采用的AUR自动生成部署：https://github.com/marketplace/actions/publish-aur-package

---

**Reviewer 注意事项**：

- 请检查 Secrets 是否正确配置
- 验证 SSH 密钥是否有 AUR 访问权限
- 确认 PKGBUILD 格式正确

---
