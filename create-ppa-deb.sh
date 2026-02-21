#!/bin/bash
# 创建可直接上传到 PPA 的 deb 包（嵌套大包）

VERSION="0.6.5"
BUILD_DIR="./ppa-deb-build"
PACKAGE_DIR="${BUILD_DIR}/sealantern_${VERSION}"

# 清理旧文件
rm -rf "$BUILD_DIR"
mkdir -p "$PACKAGE_DIR/DEBIAN"
mkdir -p "$PACKAGE_DIR/opt/sealantern"

# 使用已经下载的 deb 包
if [ -f "Sea.Lantern_${VERSION}_amd64.deb" ]; then
    cp "Sea.Lantern_${VERSION}_amd64.deb" "$PACKAGE_DIR/opt/sealantern/"
else
    echo "错误：找不到 Sea.Lantern_${VERSION}_amd64.deb"
    echo "请先下载："
    echo "wget https://github.com/SeaLantern-Studio/SeaLantern/releases/download/sea-lantern-v${VERSION}/Sea.Lantern_${VERSION}_amd64.deb"
    exit 1
fi

# 创建 control 文件（一次性写入，避免重复）
TOTAL_SIZE=$(du -sk "$PACKAGE_DIR" | cut -f1)

cat > "${PACKAGE_DIR}/DEBIAN/control" << EOF
Package: sea-lantern-ppa-updater
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Depends: dpkg, binutils, libwebkit2gtk-4.1-0, libgtk-3-0, libappindicator3-1, libssl3, ca-certificates
Installed-Size: ${TOTAL_SIZE}
Maintainer: Brian Ikun <2914651630@qq.com>
Homepage: https://github.com/SeaLantern-Studio/SeaLantern
Description: Sea Lantern PPA Installer
 This package downloads and installs Sea Lantern from official GitHub Releases.
 It's designed for PPA distribution to provide easy installation of Sea Lantern.
 .
 Sea Lantern is a lightweight Minecraft server management tool
 based on Tauri 2 + Rust + Vue 3.
EOF

# 创建 postinst 脚本 - 直接安装 Sea Lantern
cat > "${PACKAGE_DIR}/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash
set -e

PACKAGE_VERSION=$(dpkg -s sea-lantern-ppa-updater 2>/dev/null | grep Version | awk '{print $2}' | sed 's/~.*$//')
if [ -z "$PACKAGE_VERSION" ]; then
    PACKAGE_VERSION="0.6.5"
fi

EMBEDDED_DEB="/opt/sealantern/Sea.Lantern_${PACKAGE_VERSION}_amd64.deb"

echo "正在安装 Sea Lantern ${PACKAGE_VERSION}..."

if [ -f "$EMBEDDED_DEB" ]; then
    # 解压文件到根目录，不使用 dpkg 安装
    ar p "$EMBEDDED_DEB" data.tar.zst 2>/dev/null | tar --zstd -x -C / 2>/dev/null || \
    ar p "$EMBEDDED_DEB" data.tar.xz 2>/dev/null | tar -xJ -C / 2>/dev/null || \
    ar p "$EMBEDDED_DEB" data.tar.gz 2>/dev/null | tar -xz -C / 2>/dev/null
    
    rm -f "$EMBEDDED_DEB"
    
    echo "✅ Sea Lantern 已安装！"
    echo "运行命令: sea-lantern"
    
    # 更新桌面数据库
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications 2>/dev/null || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi
else
    echo "❌ 找不到嵌入的包文件: $EMBEDDED_DEB"
fi
POSTINST

chmod +x "${PACKAGE_DIR}/DEBIAN/postinst"

# 创建 prerm 脚本
cat > "${PACKAGE_DIR}/DEBIAN/prerm" << 'PRERM'
#!/bin/bash
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo "正在卸载 Sea Lantern..."
    
    # 停止运行中的进程
    killall sea-lantern 2>/dev/null || true
    
    # 删除 sealantern 的所有文件
    rm -f /usr/bin/sea-lantern
    rm -rf /usr/share/icons/hicolor/*/apps/sea-lantern.png
    rm -f /usr/share/applications/Sea\ Lantern.desktop
    
    # 删除临时文件
    rm -f /tmp/sealantern-install.log
    
    # 删除配置目录
    rm -rf ~/.config/sea-lantern 2>/dev/null || true
    
    # 更新桌面数据库
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications 2>/dev/null || true
    fi
    # 更新图标缓存
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi
    
    echo "✅ Sea Lantern 文件已清理"
fi
PRERM

chmod +x "${PACKAGE_DIR}/DEBIAN/prerm"

# 构建 deb 包
dpkg-deb --build "$PACKAGE_DIR"

FINAL_DEB="${PACKAGE_DIR}.deb"
FILE_SIZE=$(ls -lh "$FINAL_DEB" | awk '{print $5}')

echo "✅ PPA package 创建完成！"
echo "文件: $FINAL_DEB"
echo "大小: $FILE_SIZE"
echo ""
echo "这个包可以直接上传到 Launchpad PPA"