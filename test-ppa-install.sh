#!/bin/bash
# PPA Deb 包安装测试脚本
# 用于在本地测试 deb 包的安装、菜单项和命令是否正常

set -e

VERSION="0.6.5"
TEST_DIR="/tmp/sealantern-test-$$"
PKG_DIR="${TEST_DIR}/pkg"

echo "========================================="
echo "Sea Lantern PPA 包安装测试"
echo "========================================="
echo ""

# 清理函数
cleanup() {
    echo ""
    echo "清理测试环境..."
    sudo rm -rf "$TEST_DIR"
    # 如果安装了测试包，卸载它
    if dpkg -l | grep -q sea-lantern-ppa-updater; then
        echo "卸载测试包..."
        sudo dpkg -r sea-lantern-ppa-updater 2>/dev/null || true
        sudo dpkg --purge sea-lantern-ppa-updater 2>/dev/null || true
    fi
    # 清理 Sea Lantern 安装的文件
    sudo rm -f /usr/bin/sea-lantern
    sudo rm -f /usr/share/applications/Sea\ Lantern.desktop
    sudo rm -rf /usr/share/icons/hicolor/*/apps/sealantern.png
    sudo rm -rf /opt/sealantern
}

# 设置清理陷阱
trap cleanup EXIT

# 创建测试目录
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/opt/sealantern"

echo "1. 准备测试包..."

# 下载原始 deb 包（如果不存在）
if [ ! -f "Sea.Lantern_${VERSION}_amd64.deb" ]; then
    echo "   下载 Sea.Lantern_${VERSION}_amd64.deb..."
    wget -q "https://github.com/SeaLantern-Studio/SeaLantern/releases/download/sea-lantern-v${VERSION}/Sea.Lantern_${VERSION}_amd64.deb" \
        -O "Sea.Lantern_${VERSION}_amd64.deb"
fi

# 复制嵌入的 deb 包
cp "Sea.Lantern_${VERSION}_amd64.deb" "$PKG_DIR/opt/sealantern/"

# 创建 control 文件
TOTAL_SIZE=$(du -sk "$PKG_DIR" | cut -f1)

cat > "${PKG_DIR}/DEBIAN/control" << EOF
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
 .
 Sea Lantern is a lightweight Minecraft server management tool
 based on Tauri 2 + Rust + Vue 3.
EOF

# 创建修复后的 postinst 脚本
cat > "${PKG_DIR}/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash
set -e

# 获取已安装的包版本号 - 使用 dpkg-query 更可靠
PACKAGE_VERSION=$(dpkg-query -W -f='${Version}' sea-lantern-ppa-updater 2>/dev/null | sed 's/~.*$//')
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
    echo "请检查安装包是否完整"
fi
POSTINST
chmod +x "${PKG_DIR}/DEBIAN/postinst"

# 创建 prerm 脚本
cat > "${PKG_DIR}/DEBIAN/prerm" << 'PRERM'
#!/bin/bash
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo "正在卸载 Sea Lantern..."

    # 停止运行中的进程
    killall sea-lantern 2>/dev/null || true

    # 删除 Sea Lantern 的文件
    rm -f /usr/bin/sea-lantern
    rm -f /usr/share/applications/Sea\ Lantern.desktop
    rm -rf /usr/share/icons/hicolor/*/apps/sealantern.png
    rm -rf /usr/share/icons/hicolor/*/apps/Sea\ Lantern.png
    rm -rf /var/lib/sealantern

    # 删除临时文件
    rm -f /opt/sealantern/install.sh

    # 更新桌面数据库
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications 2>/dev/null || true
    fi
    # 更新图标缓存
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi

    echo "✅ Sea Lantern 文件已删除"
fi
PRERM
chmod +x "${PKG_DIR}/DEBIAN/prerm"

# 构建测试包
echo "2. 构建测试包..."
dpkg-deb --build "$PKG_DIR" "${TEST_DIR}/test-package.deb" > /dev/null 2>&1
echo "   ✅ 测试包构建完成"

echo ""
echo "3. 安装测试包..."
sudo dpkg -i "${TEST_DIR}/test-package.deb"

echo ""
echo "4. 验证安装结果..."

# 检查命令是否存在
if [ -f "/usr/bin/sea-lantern" ]; then
    echo "   ✅ /usr/bin/sea-lantern 存在"
else
    echo "   ❌ /usr/bin/sea-lantern 不存在"
fi

# 检查桌面文件是否存在
if [ -f "/usr/share/applications/Sea Lantern.desktop" ]; then
    echo "   ✅ /usr/share/applications/Sea Lantern.desktop 存在"
    # 检查 Exec 行
    if grep -q "Exec=/usr/bin/sealantern" "/usr/share/applications/Sea Lantern.desktop"; then
        echo "   ✅ Exec 命令配置正确"
    else
        echo "   ⚠️  Exec 命令可能需要检查"
        grep "Exec=" "/usr/share/applications/Sea Lantern.desktop"
    fi
else
    echo "   ❌ /usr/share/applications/Sea Lantern.desktop 不存在"
fi

# 检查图标是否存在
ICON_FOUND=false
for size in 32x32 64x64 128x128 256x256; do
    if [ -f "/usr/share/icons/hicolor/${size}/apps/sealantern.png" ]; then
        echo "   ✅ 图标 ${size} 存在"
        ICON_FOUND=true
    fi
done
if [ "$ICON_FOUND" = false ]; then
    echo "   ⚠️  未找到图标文件"
fi

# 检查嵌入的 deb 是否被删除
if [ -f "/opt/sealantern/Sea.Lantern_${VERSION}_amd64.deb" ]; then
    echo "   ⚠️  嵌入的 deb 包未被删除"
else
    echo "   ✅ 嵌入的 deb 包已清理"
fi

echo ""
echo "5. 测试命令执行..."
if command -v sea-lantern >/dev/null 2>&1; then
    echo "   ✅ sea-lantern 命令可用"
    # 检查版本
    sea-lantern --version 2>/dev/null || echo "   ⚠️  无法获取版本（可能需要图形环境）"
else
    echo "   ❌ sea-lantern 命令不可用"
fi

echo ""
echo "========================================="
echo "测试完成！"
echo "========================================="
echo ""
echo "清理中..."
