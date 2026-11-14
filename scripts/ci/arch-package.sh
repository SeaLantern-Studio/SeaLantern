#!/usr/bin/env bash

# 渲染 / 构建 Arch Linux 安装包（pkg.tar.zst）。
#
#   render <version> <tag> <output-dir>
#       渲染 AUR 发布用的 PKGBUILD、sealantern.desktop 与 sealantern.install，
#       交给 AUR 工作流生成 .SRCINFO 并推送。
#   build <version> <deb-path> <output-dir>
#       把 Release 用的 x64 deb 重新打包为 sealantern-<version>-1-x86_64.pkg.tar.zst，
#       供 Release 直接下载安装，保持与 AUR 配方一致。
#
# build 模式依赖 makepkg，必须在非 root 的 Arch Linux（或 archlinux 容器）中执行。

set -euo pipefail

readonly script_dir="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly repo_dir="$(cd -- "${script_dir}/../.." && pwd)"
readonly packaging_dir="${repo_dir}/packaging"
readonly template_path="${packaging_dir}/PKGBUILD.in"
readonly release_repo_url="https://github.com/SeaLantern-Studio/SeaLantern"

usage() {
    echo "usage: arch-package.sh render <version> <tag> <output-dir>" >&2
    echo "       arch-package.sh build <version> <deb-path> <output-dir>" >&2
    exit 2
}

die() {
    echo "$*" >&2
    exit 1
}

validate_version() {
    if [[ ! "$1" =~ ^[0-9A-Za-z.+-]+$ ]]; then
        echo "invalid version: $1" >&2
        exit 2
    fi
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "required command not found: $1"
    fi
}

sed_escape() {
    printf '%s' "$1" | sed -e 's/[\\&|]/\\&/g'
}

render_pkgbuild() {
    local version="$1"
    local deb_source="$2"
    local deb_sha256="$3"
    local output_dir="$4"

    if [[ ! -f "${template_path}" ]]; then
        die "PKGBUILD template not found: ${template_path}"
    fi

    mkdir -p "${output_dir}"
    sed \
        -e "s|@PKGVER@|$(sed_escape "${version}")|g" \
        -e "s|@DEB_SOURCE@|$(sed_escape "${deb_source}")|g" \
        -e "s|@DEB_SHA256@|$(sed_escape "${deb_sha256}")|g" \
        "${template_path}" >"${output_dir}/PKGBUILD"

    if grep -Eq '@[A-Z_]+@' "${output_dir}/PKGBUILD"; then
        die "rendered PKGBUILD still contains template placeholders"
    fi

    install -m644 "${packaging_dir}/sealantern.desktop" "${packaging_dir}/sealantern.install" \
        "${output_dir}/"
}

render_aur_package() {
    local version="$1"
    local tag="$2"
    local output_dir="$3"

    if [[ ! "${tag}" =~ ^[0-9A-Za-z._+-]+$ ]]; then
        echo "invalid tag: ${tag}" >&2
        exit 2
    fi

    # AUR 直接从 Release 下载 x64 deb，再解包成 Arch 文件布局。
    local deb_source
    deb_source="\"sealantern-\${pkgver}-\${_debarch}.deb::${release_repo_url}/releases/download/${tag}/Sea.Lantern_\${pkgver}_\${_debarch}.deb\""
    render_pkgbuild "${version}" "${deb_source}" "SKIP" "${output_dir}"

    echo "rendered AUR packaging files for ${version} into ${output_dir}"
}

verify_package() {
    local package_path="$1"
    local expected_name="$2"
    local package_name
    package_name="$(basename "${package_path}")"

    if [[ "${package_name}" != "${expected_name}" ]]; then
        die "unexpected package file name: ${package_name} (expected ${expected_name})"
    fi
    if [[ ! -s "${package_path}" ]]; then
        die "package is empty: ${package_path}"
    fi

    local listing
    listing="$(bsdtar -tf "${package_path}")"
    for expected_file in \
        '.PKGINFO' \
        '.MTREE' \
        'usr/bin/sealantern' \
        'usr/share/applications/sealantern.desktop' \
        'usr/share/icons/hicolor/128x128/apps/sealantern.png'; do
        if ! grep -qx -- "${expected_file}" <<<"${listing}"; then
            die "package is missing ${expected_file}: ${package_path}"
        fi
    done
}

build_arch_package() {
    local version="$1"
    local deb_path="$2"
    local output_dir="$3"

    if [[ "${EUID}" -eq 0 ]]; then
        die "makepkg refuses to run as root; run build mode as a regular user"
    fi
    if [[ "$(uname -m)" != "x86_64" ]]; then
        die "only x86_64 packages are built, current machine: $(uname -m)"
    fi
    if [[ ! -f "${deb_path}" ]]; then
        die "deb not found: ${deb_path}"
    fi

    for command_name in bsdtar makepkg sha256sum install; do
        require_command "${command_name}"
    done

    local work_dir
    work_dir="$(mktemp -d "${TMPDIR:-/tmp}/sealantern-arch.XXXXXX")"
    # shellcheck disable=SC2064
    trap "rm -rf -- '${work_dir}'" EXIT

    local deb_name="sealantern-${version}-amd64.deb"
    install -m644 "${deb_path}" "${work_dir}/${deb_name}"

    local deb_sha256
    deb_sha256="$(sha256sum "${work_dir}/${deb_name}" | cut -d' ' -f1)"
    render_pkgbuild "${version}" "\"sealantern-\${pkgver}-\${_debarch}.deb\"" "${deb_sha256}" "${work_dir}"

    (
        cd "${work_dir}"
        PKGDEST="${work_dir}" makepkg --noconfirm --nodeps --force
    )

    local expected_name="sealantern-${version}-1-x86_64.pkg.tar.zst"
    local -a packages=()
    mapfile -t packages < <(find "${work_dir}" -maxdepth 1 -type f -name '*.pkg.tar.zst' | sort)
    if [[ "${#packages[@]}" -ne 1 ]]; then
        echo "expected exactly one built package, found ${#packages[@]}" >&2
        printf '  %s\n' "${packages[@]}" >&2
        exit 1
    fi
    verify_package "${packages[0]}" "${expected_name}"

    mkdir -p "${output_dir}"
    install -m644 "${packages[0]}" "${output_dir}/${expected_name}"
    echo "built ${output_dir}/${expected_name}"
}

if [[ "$#" -ne 4 ]]; then
    usage
fi

mode="$1"
version="$2"
validate_version "${version}"

case "${mode}" in
    render)
        render_aur_package "${version}" "$3" "$4"
        ;;
    build)
        build_arch_package "${version}" "$3" "$4"
        ;;
    *)
        usage
        ;;
esac
