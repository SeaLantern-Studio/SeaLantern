#!/bin/bash
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo "正在卸载 Sea Lantern..."

    # 删除 Sea Lantern 的文件
    rm -f /usr/bin/sea-lantern
    rm -f /usr/share/applications/Sea\ Lantern.desktop
    rm -rf /usr/share/icons/hicolor/*/apps/sea-lantern.png
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
