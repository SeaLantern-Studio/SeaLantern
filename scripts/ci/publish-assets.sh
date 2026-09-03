#!/usr/bin/env bash

set -euo pipefail

if [[ "$#" -ne 7 ]]; then
    echo "usage: publish-assets.sh <assets-dir> <tag> <release-name> <source-sha> <draft> <prerelease> <cleanup-tag-on-failure>" >&2
    exit 2
fi

readonly assets_dir="$1"
readonly tag="$2"
readonly release_name="$3"
readonly source_sha="$4"
readonly draft="$5"
readonly prerelease="$6"
readonly cleanup_tag_on_failure="$7"

if [[ "${draft}" != "true" && "${draft}" != "false" ]]; then
    echo "draft must be true or false" >&2
    exit 2
fi
if [[ "${prerelease}" != "true" && "${prerelease}" != "false" ]]; then
    echo "prerelease must be true or false" >&2
    exit 2
fi
if [[ "${cleanup_tag_on_failure}" != "true" && "${cleanup_tag_on_failure}" != "false" ]]; then
    echo "cleanup-tag-on-failure must be true or false" >&2
    exit 2
fi

for command_name in basename find gh grep jq mktemp rm sed sort uniq wc; do
    if ! command -v "${command_name}" >/dev/null 2>&1; then
        echo "required command not found: ${command_name}" >&2
        exit 1
    fi
done
if [[ ! -d "${assets_dir}" ]]; then
    echo "assets directory not found: ${assets_dir}" >&2
    exit 1
fi
if [[ -z "${GH_TOKEN:-}" || -z "${MATRIX_JSON:-}" ]]; then
    echo "GH_TOKEN and MATRIX_JSON must be set" >&2
    exit 1
fi
if ! jq -e '.include | type == "array" and length > 0' <<<"${MATRIX_JSON}" >/dev/null; then
    echo "MATRIX_JSON does not contain a valid include matrix" >&2
    exit 1
fi

mapfile -t expected_keys < <(jq -r '.include[].asset_key' <<<"${MATRIX_JSON}" | sort)
if [[ "${#expected_keys[@]}" -ne "$(printf '%s\n' "${expected_keys[@]}" | uniq | wc -l)" ]]; then
    echo "the platform matrix contains duplicate asset keys" >&2
    exit 1
fi

for asset_key in "${expected_keys[@]}"; do
    artifact_dir="${assets_dir}/release-assets-${asset_key}"
    if [[ ! -d "${artifact_dir}" ]]; then
        echo "missing workflow artifact directory: release-assets-${asset_key}" >&2
        exit 1
    fi
    if ! find "${artifact_dir}" -maxdepth 1 -type f -print -quit | grep -q .; then
        echo "workflow artifact is empty: release-assets-${asset_key}" >&2
        exit 1
    fi
done

while IFS= read -r artifact_dir; do
    actual_key="$(basename -- "${artifact_dir}")"
    actual_key="${actual_key#release-assets-}"
    if ! jq -e --arg key "${actual_key}" '.include | any(.asset_key == $key)' \
        <<<"${MATRIX_JSON}" >/dev/null; then
        echo "unexpected workflow artifact directory: $(basename -- "${artifact_dir}")" >&2
        exit 1
    fi
done < <(find "${assets_dir}" -mindepth 1 -maxdepth 1 -type d -name 'release-assets-*' | sort)

mapfile -t assets < <(find "${assets_dir}" -mindepth 2 -maxdepth 2 -type f | sort)
if [[ "${#assets[@]}" -eq 0 ]]; then
    echo "no release assets were found" >&2
    exit 1
fi

duplicates="$(printf '%s\n' "${assets[@]##*/}" | sort | uniq -d)"
if [[ -n "${duplicates}" ]]; then
    echo "duplicate release asset names were found:" >&2
    while IFS= read -r duplicate; do
        [[ -z "${duplicate}" ]] && continue
        echo "  ${duplicate}" >&2
        find "${assets_dir}" -type f -name "${duplicate}" -print | sed 's/^/    /' >&2
    done <<<"${duplicates}"
    exit 1
fi

notes_file="$(mktemp "${RUNNER_TEMP:-/tmp}/release-notes.XXXXXX.md")"
cleanup() {
    rm -f -- "${notes_file}"
}
trap cleanup EXIT
printf '%s\n' "${RELEASE_BODY:-}" >"${notes_file}"

printf 'publishing %s assets:\n' "${#assets[@]}"
printf '  %s\n' "${assets[@]}"

if release_json="$(gh release view "${tag}" --json isDraft 2>/dev/null)"; then
    existing_draft="$(jq -r '.isDraft' <<<"${release_json}")"
    if [[ "${draft}" == "true" && "${existing_draft}" != "true" ]]; then
        echo "refusing to turn an existing published release back into a draft: ${tag}" >&2
        exit 1
    fi

    gh release edit "${tag}" \
        --title "${release_name}" \
        --notes-file "${notes_file}" \
        --prerelease="${prerelease}"
    gh release upload "${tag}" "${assets[@]}" --clobber
    if [[ "${existing_draft}" == "true" && "${draft}" == "false" ]]; then
        gh release edit "${tag}" --draft=false --prerelease="${prerelease}"
    fi
else
    release_flags=()
    if [[ "${draft}" == "true" ]]; then
        release_flags+=(--draft)
    fi
    if [[ "${prerelease}" == "true" ]]; then
        release_flags+=(--prerelease)
    fi
    if ! gh release create "${tag}" "${assets[@]}" \
        --target "${source_sha}" \
        --title "${release_name}" \
        --notes-file "${notes_file}" \
        "${release_flags[@]}"; then
        echo "release creation failed; removing any partial release or tag for ${tag}" >&2
        cleanup_flags=(-y)
        if [[ "${cleanup_tag_on_failure}" == "true" ]]; then
            cleanup_flags+=(--cleanup-tag)
        fi
        gh release delete "${tag}" "${cleanup_flags[@]}" >/dev/null 2>&1 || true
        exit 1
    fi
fi
