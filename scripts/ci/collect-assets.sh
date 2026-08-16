#!/usr/bin/env bash

set -euo pipefail

if [[ "$#" -ne 4 ]]; then
    echo "usage: collect-assets.sh <linux|macos> <version> <asset-arch> <asset-key>" >&2
    exit 2
fi

readonly platform="$1"
readonly version="$2"
readonly asset_arch="$3"
readonly asset_key="$4"
readonly script_dir="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly repo_dir="$(cd -- "${script_dir}/../.." && pwd)"

if [[ "${platform}" != "linux" && "${platform}" != "macos" ]]; then
    echo "unsupported platform: ${platform}" >&2
    exit 2
fi
if [[ ! "${version}" =~ ^[0-9A-Za-z.+-]+$ ]]; then
    echo "invalid version: ${version}" >&2
    exit 2
fi
if [[ ! "${asset_arch}" =~ ^(x64|arm64)$ ]]; then
    echo "invalid asset architecture: ${asset_arch}" >&2
    exit 2
fi
if [[ ! "${asset_key}" =~ ^[a-z0-9-]+$ ]]; then
    echo "invalid asset key: ${asset_key}" >&2
    exit 2
fi
if [[ -z "${RUNNER_TEMP:-}" ]]; then
    echo "RUNNER_TEMP is not set" >&2
    exit 1
fi

readonly asset_dir="${RUNNER_TEMP}/release-assets-${asset_key}"
readonly portable_dir="${RUNNER_TEMP}/portable-${asset_key}"
search_roots=()

for candidate in "${repo_dir}/target" "${repo_dir}/src-tauri/target"; do
    if [[ -d "${candidate}" ]]; then
        search_roots+=("${candidate}")
    fi
done
if [[ "${#search_roots[@]}" -eq 0 ]]; then
    echo "no target directory was found" >&2
    exit 1
fi
if [[ -e "${asset_dir}" || -e "${portable_dir}" ]]; then
    echo "staging directory already exists for ${asset_key}" >&2
    exit 1
fi
mkdir -p "${asset_dir}" "${portable_dir}"

stage_asset() {
    local source_path="$1"
    local source_name="$(basename "${source_path}")"
    local destination_path="${asset_dir}/${source_name}"

    if [[ ! -f "${source_path}" ]]; then
        echo "asset is not a regular file: ${source_path}" >&2
        exit 1
    fi
    case "${asset_arch}" in
        x64)
            if [[ ! "${source_name}" =~ (x64|amd64|x86_64) ]]; then
                echo "x64 asset name is missing an architecture marker: ${source_name}" >&2
                exit 1
            fi
            ;;
        arm64)
            if [[ ! "${source_name}" =~ (arm64|aarch64) ]]; then
                echo "ARM64 asset name is missing an architecture marker: ${source_name}" >&2
                exit 1
            fi
            ;;
    esac
    if [[ -e "${destination_path}" ]]; then
        echo "duplicate staged asset name: ${source_name}" >&2
        exit 1
    fi
    cp "${source_path}" "${destination_path}"
}

find_unique_file() {
    local description="$1"
    local name_pattern="$2"
    local path_pattern="$3"
    local require_version="${4:-true}"
    local candidate
    local -a matches=()

    while IFS= read -r -d '' candidate; do
        if [[ "${require_version}" == "false" || "$(basename "${candidate}")" == *"${version}"* ]]; then
            matches+=("${candidate}")
        fi
    done < <(
        find "${search_roots[@]}" -type f -name "${name_pattern}" -path "${path_pattern}" -print0 2>/dev/null
    )

    if [[ "${#matches[@]}" -ne 1 ]]; then
        echo "expected exactly one ${description} for version ${version}, found ${#matches[@]}" >&2
        if [[ "${#matches[@]}" -gt 0 ]]; then
            printf '  %s\n' "${matches[@]}" >&2
        fi
        exit 1
    fi
    unique_file="${matches[0]}"
}

collect_linux_assets() {
    find_unique_file "AppImage bundle" "*.AppImage" "*/bundle/appimage/*"
    stage_asset "${unique_file}"
    find_unique_file "Debian bundle" "*.deb" "*/bundle/deb/*"
    stage_asset "${unique_file}"
    find_unique_file "RPM bundle" "*.rpm" "*/bundle/rpm/*"
    stage_asset "${unique_file}"
}

find_unique_app() {
    local candidate
    local -a apps=()

    while IFS= read -r -d '' candidate; do
        apps+=("${candidate}")
    done < <(find "${search_roots[@]}" -type d -name '*.app' -path '*/bundle/macos/*.app' -print0 2>/dev/null)

    if [[ "${#apps[@]}" -eq 0 ]]; then
        return 1
    fi
    if [[ "${#apps[@]}" -gt 1 ]]; then
        echo "expected exactly one macOS app bundle, found ${#apps[@]}" >&2
        printf '  %s\n' "${apps[@]}" >&2
        exit 1
    fi
    unique_app="${apps[0]}"
}

collect_macos_assets() {
    local app_basename
    local candidate
    local extracted_app
    local portable_path
    local root_app
    local -a extracted_apps=()

    find_unique_file "macOS DMG bundle" "*.dmg" "*/bundle/dmg/*"
    stage_asset "${unique_file}"

    if find_unique_app; then
        cp -R "${unique_app}" "${portable_dir}/"
        app_basename="$(basename "${unique_app}")"
    else
        find_unique_file "macOS app archive" "*.app.tar.gz" "*/bundle/macos/*" false
        tar -C "${portable_dir}" -xzf "${unique_file}"
        while IFS= read -r -d '' candidate; do
            extracted_apps+=("${candidate}")
        done < <(find "${portable_dir}" -type d -name '*.app' -prune -print0)
        if [[ "${#extracted_apps[@]}" -ne 1 ]]; then
            echo "expected exactly one app bundle in the macOS app archive, found ${#extracted_apps[@]}" >&2
            if [[ "${#extracted_apps[@]}" -gt 0 ]]; then
                printf '  %s\n' "${extracted_apps[@]}" >&2
            fi
            exit 1
        fi

        extracted_app="${extracted_apps[0]}"
        app_basename="$(basename "${extracted_app}")"
        root_app="${portable_dir}/${app_basename}"
        if [[ "${extracted_app}" != "${root_app}" ]]; then
            if [[ -e "${root_app}" ]]; then
                echo "cannot normalize the extracted app bundle because ${root_app} already exists" >&2
                exit 1
            fi
            mv "${extracted_app}" "${root_app}"
        fi
    fi

    cp "${repo_dir}/LICENSE" "${repo_dir}/NOTICE" "${portable_dir}/"
    portable_path="${asset_dir}/Sea.Lantern_${version}_macos_${asset_arch}_portable.tar.gz"
    tar -C "${portable_dir}" -czf "${portable_path}" "${app_basename}" LICENSE NOTICE
}

cd "${repo_dir}"
case "${platform}" in
    linux) collect_linux_assets ;;
    macos) collect_macos_assets ;;
esac

echo "staged ${platform} ${asset_arch} assets:"
find "${asset_dir}" -maxdepth 1 -type f -print | sort
