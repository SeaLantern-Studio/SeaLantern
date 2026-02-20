# Sea Lantern Ubuntu PPA 自动发布

这个分支专门用于自动同步上游 Sea Lantern 发布并上传到 Ubuntu PPA。

## PPA 地址

```bash
sudo add-apt-repository ppa:brianeee7878/sealantern
sudo apt update
sudo apt install sealantern
```

## 工作原理

1. **自动检测**：每6小时检查一次上游是否有新 Release
2. **自动下载**：从上游 GitHub Releases 下载 deb 包
3. **自动转换**：将 deb 包转换为 Debian 源代码包
4. **自动签名**：使用 GPG 密钥签名
5. **自动上传**：上传到 Launchpad PPA

## 手动触发

在 GitHub Actions 页面点击 "Run workflow"，可以：
- 自动检测最新版本
- 或手动指定版本号

## 上游仓库

- **Source**: https://github.com/SeaLantern-Studio/SeaLantern
- **License**: GPL-3.0

## 维护者

- Brian Ikun <2914651630@qq.com>
- PPA: ppa:brianeee7878/sealantern
