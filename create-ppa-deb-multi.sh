#!/bin/bash
# 创建可直接上传到 PPA 的 deb 包（嵌套大包），支持多个 Ubuntu 版本

VERSION="0.6.5"
BUILD_DIR="./ppa-deb-build-multi"

# 支持的 Ubuntu 版本（从新到旧）
UBUNTU_VERSIONS=("noble" "jammy" "focal")

# 清理旧文件
rm -rf "$BUILD_DIR"

for DISTRO in "${UBUNTU_VERSIONS[@]}"; do
    echo "正在构建 ${DISTRO} 版本..."
    
    # 添加版本后缀（~distro1 表示向后兼容）
    PACKAGE_VERSION="${VERSION}~${DISTRO}1"
    PACKAGE_DIR="${BUILD_DIR}/sea-lantern-ppa-updater_${PACKAGE_VERSION}"
    
    mkdir -p "$PACKAGE_DIR/DEBIAN"
    mkdir -p "$PACKAGE_DIR/opt/sealantern"
    
    # 使用已经下载的 deb 包
    if [ -f "Sea.Lantern_${VERSION}_amd64.deb" ]; then
        cp "Sea.Lantern_${VERSION}_amd64.deb" "$PACKAGE_DIR/opt/sealantern/"
    else
        echo "正在下载 Sea.Lantern_${VERSION}_amd64.deb..."
        wget -O "Sea.Lantern_${VERSION}_amd64.deb" \
            "https://github.com/SeaLantern-Studio/SeaLantern/releases/download/sea-lantern-v${VERSION}/Sea.Lantern_${VERSION}_amd64.deb"
        cp "Sea.Lantern_${VERSION}_amd64.deb" "$PACKAGE_DIR/opt/sealantern/"
    fi
    
    # 创建 control 文件
    TOTAL_SIZE=$(du -sk "$PACKAGE_DIR" | cut -f1)
    
    cat > "${PACKAGE_DIR}/DEBIAN/control" << EOF
Package: sea-lantern-ppa-updater
Version: ${PACKAGE_VERSION}
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
    
    # 创建 postinst 脚本
    cat > "${PACKAGE_DIR}/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash

PACKAGE_VERSION=$(dpkg -s sea-lantern-ppa-updater 2>/dev/null | grep Version | awk '{print $2}' | sed 's/~.*$//')
if [ -z "$PACKAGE_VERSION" ]; then
    PACKAGE_VERSION="0.6.5"
fi

EMBEDDED_DEB="/opt/sealantern/Sea.Lantern_${PACKAGE_VERSION}_amd64.deb"

echo "正在安装 Sea Lantern ${PACKAGE_VERSION}..."

if [ -f "$EMBEDDED_DEB" ]; then
    dpkg -i "$EMBEDDED_DEB" || true
    
    # 安装依赖
    if ! dpkg -s sea-lantern >/dev/null 2>&1; then
        echo "正在安装依赖..."
        apt-get install -f -y
    fi
    
    if dpkg -s sea-lantern >/dev/null 2>&1; then
        echo "✅ Sea Lantern 安装完成！"
        echo "运行命令: sea-lantern"
        rm -f "$EMBEDDED_DEB"
    else
        echo "❌ 安装失败，请手动检查"
        exit 1
    fi
else
    echo "❌ 找不到嵌入的包文件: $EMBEDDED_DEB"
    exit 1
fi
POSTINST
    
    chmod +x "${PACKAGE_DIR}/DEBIAN/postinst"
    
    # 创建 prerm 脚本
    cat > "${PACKAGE_DIR}/DEBIAN/prerm" << 'PRERM'
#!/bin/bash
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo "正在卸载 Sea Lantern..."
    dpkg -l | grep -q sea-lantern && dpkg -r sea-lantern || true
fi
PRERM
    
    chmod +x "${PACKAGE_DIR}/DEBIAN/prerm"
    
    # 构建 deb 包
    dpkg-deb --build "$PACKAGE_DIR"
    
    FINAL_DEB="${PACKAGE_DIR}.deb"
    FILE_SIZE=$(ls -lh "$FINAL_DEB" | awk '{print $5}')
    
    echo "✅ ${DISTRO} 版本创建完成: $FINAL_DEB ($FILE_SIZE)"
done

echo ""
echo "========================================="
echo "所有版本构建完成！"
echo "========================================="
echo "生成的包："
ls -lh "${BUILD_DIR}"/*.deb
echo ""
echo "上传到 Launchpad PPA："
echo "dput ppa:brianeee7878/sealantern sea-lantern-ppa-updater_${VERSION}~*.changes"
