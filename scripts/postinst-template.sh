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
