# Maintainer: xuezhajv <liaozecheng123@163.com>  qq群：293748695
# Contributor: github.com/FPSZ <
pkgname=sealantern
pkgver=1.2.0
pkgrel=2
_binrel=1
pkgdesc="A lightweight Minecraft server management tool based on Tauri 2 + Rust + Vue 3"
arch=('x86_64')
url="https://github.com/SeaLantern-Studio/SeaLantern"
license=('GPL-3.0-or-later')
depends=(
    'gtk3'
    'webkit2gtk-4.1'
    'libayatana-appindicator'
    'openssl'
)

# 可选依赖（推荐但不是必须的）
optdepends=(
    'ffmpeg: 视频/音频编解码支持'
    'gst-plugins-base: GStreamer 基础插件'
    'gst-plugins-good: GStreamer 优质插件'
    'gst-plugins-bad: GStreamer 额外插件'
    'gst-plugins-ugly: GStreamer 非自由插件'
    'noto-fonts: 更好的字体显示'
    'noto-fonts-cjk: 中日韩字体支持'
    'noto-fonts-emoji: 表情符号支持'
)

options=('!strip' '!emptydirs')
install=sealantern.install

source=(
    "${pkgname}-${pkgver}-${_binrel}-${CARCH}.release.pkg.tar.zst::https://github.com/SeaLantern-Studio/SeaLantern/releases/download/v${pkgver}/${pkgname}-${pkgver}-${_binrel}-${CARCH}.pkg.tar.zst"
    'sealantern.desktop'
)
sha256sums=(
    'SKIP'
    'SKIP'
)

package() {
    local archive="${srcdir}/${pkgname}-${pkgver}-${_binrel}-${CARCH}.release.pkg.tar.zst"

    bsdtar --no-same-owner -xpf "${archive}" -C "${pkgdir}"
    rm -f "${pkgdir}/.BUILDINFO" \
          "${pkgdir}/.INSTALL" \
          "${pkgdir}/.MTREE" \
          "${pkgdir}/.PKGINFO"

    # 兼容上游可能将可执行文件放在 /usr/local/bin 的情况
    if [[ -f "${pkgdir}/usr/local/bin/sea-lantern" ]]; then
        install -dm755 "${pkgdir}/usr/bin"
        mv "${pkgdir}/usr/local/bin/sea-lantern" "${pkgdir}/usr/bin/"
        rmdir "${pkgdir}/usr/local/bin" 2>/dev/null || true
        rmdir "${pkgdir}/usr/local" 2>/dev/null || true
    fi

    # 确保图标缓存更新钩子能工作
    if [[ -d "${pkgdir}/usr/share/icons" ]]; then
        find "${pkgdir}/usr/share/icons" -name "icon-theme.cache" -delete
    fi

    rm -f "${pkgdir}/usr/share/applications/Sea Lantern.desktop"
    install -Dm644 "${srcdir}/sealantern.desktop" \
        "${pkgdir}/usr/share/applications/sealantern.desktop"
}
