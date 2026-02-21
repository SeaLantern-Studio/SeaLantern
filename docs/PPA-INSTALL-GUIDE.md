# Sea Lantern PPA 安装指南

## 快速安装

### 1. 添加 PPA 源

```bash
sudo add-apt-repository ppa:brianeee7878/sealantern
sudo apt update
```

### 2. 安装 Sea Lantern

```bash
sudo apt install sea-lantern-ppa-updater
```

安装过程中会自动下载并安装 Sea Lantern 主程序。

### 3. 启动 Sea Lantern

```bash
# 命令行启动
sea-lantern

# 或者在应用菜单中搜索 "Sea Lantern"
```

---

## 支持的 Ubuntu 版本

| Ubuntu 版本 | 代号 | 支持状态 |
|------------|------|---------|
| Ubuntu 24.04 LTS | noble | ✅ 支持 |
| Ubuntu 22.04 LTS | jammy | ✅ 支持 |
| Ubuntu 20.04 LTS | focal | ✅ 支持 |

---

## 详细安装步骤

### 图形界面安装（推荐新手）

1. **打开"软件和更新"**
   - 在应用菜单中搜索 "Software & Updates"
   - 或者使用快捷键 `Super` 键，然后输入 "software"

2. **添加 PPA**
   - 点击 "Other Software" 标签
   - 点击 "Add..." 按钮
   - 输入：`ppa:brianeee7878/sealantern`
   - 点击 "Add Source"
   - 输入密码确认

3. **安装软件**
   - 打开 "Ubuntu Software"
   - 搜索 "Sea Lantern"
   - 点击安装

### 命令行安装

```bash
# 添加 PPA
sudo add-apt-repository ppa:brianeee7878/sealantern -y

# 更新软件源
sudo apt update

# 安装
sudo apt install sea-lantern-ppa-updater -y

# 验证安装
sea-lantern --version
```

---

## 卸载

### 完全卸载

```bash
# 卸载 PPA 包（会自动清理 Sea Lantern）
sudo apt remove --purge sea-lantern-ppa-updater

# 可选：删除 PPA 源
sudo add-apt-repository --remove ppa:brianeee7878/sealantern
```

### 仅卸载 Sea Lantern，保留 PPA

```bash
sudo apt remove sea-lantern
```

---

## 故障排除

### 1. 添加 PPA 失败

**问题**：`add-apt-repository: command not found`

**解决**：
```bash
sudo apt install software-properties-common
```

### 2. 安装后找不到命令

**问题**：`sea-lantern: command not found`

**解决**：
```bash
# 检查是否安装成功
dpkg -l | grep sea-lantern

# 如果已安装但找不到命令，尝试重新登录或重启
# 或者手动创建符号链接
sudo ln -sf /usr/bin/sealantern /usr/local/bin/sea-lantern
```

### 3. 桌面图标不显示

**解决**：
```bash
# 更新桌面数据库
sudo update-desktop-database /usr/share/applications

# 更新图标缓存
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
```

### 4. 依赖问题

**问题**：安装时出现依赖错误

**解决**：
```bash
# 修复依赖
sudo apt install -f

# 然后重新安装
sudo apt install sea-lantern-ppa-updater
```

---

## 更新 Sea Lantern

当有新版本发布时，PPA 会自动提供更新：

```bash
# 更新所有软件
sudo apt update && sudo apt upgrade

# 或者只更新 Sea Lantern
sudo apt update && sudo apt install sea-lantern-ppa-updater
```

---

## 系统要求

- **操作系统**：Ubuntu 20.04 LTS 或更高版本
- **架构**：amd64 (x86_64)
- **内存**：至少 4GB RAM
- **磁盘空间**：至少 200MB 可用空间
- **依赖**：
  - libwebkit2gtk-4.1-0
  - libgtk-3-0
  - libappindicator3-1
  - libssl3

---

## PPA 信息

- **PPA 地址**：`ppa:brianeee7878/sealantern`
- **维护者**：Brian Ikun
- **GitHub**：https://github.com/SeaLantern-Studio/SeaLantern
- **许可证**：GPL-3.0

---

## 手动构建（高级用户）

如果你想自己构建 deb 包：

```bash
# 克隆仓库
git clone https://github.com/SeaLantern-Studio/SeaLantern.git
cd SeaLantern

# 切换到 ppa-deploy 分支
git checkout ppa-deploy

# 运行构建脚本
./create-ppa-deb.sh

# 安装生成的包
sudo dpkg -i ./ppa-deb-build/sealantern_*.deb
```

---

## 获取帮助

- **GitHub Issues**：https://github.com/SeaLantern-Studio/SeaLantern/issues
- **QQ 群**：搜索 "Sea Lantern" 或查看 GitHub README

---

*最后更新：2026-02-21*
