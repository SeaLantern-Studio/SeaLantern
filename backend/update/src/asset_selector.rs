static WINDOWS_SUFFIXES: &[&str] = &[".msi", ".exe"];
static MACOS_SUFFIXES: &[&str] = &[".dmg", ".app", ".tar.gz"];
static LINUX_SUFFIXES: &[&str] = &[".appimage", ".deb", ".rpm", ".tar.gz"];

static X64_ARCH_KEYWORDS: &[&str] = &["x86_64", "x64", "amd64"];
static ARM64_ARCH_KEYWORDS: &[&str] = &["aarch64", "arm64", "arm"];
static EMPTY_ARCH_KEYWORDS: &[&str] = &[];
static KNOWN_ARCH_GROUPS: &[&[&str]] = &[X64_ARCH_KEYWORDS, ARM64_ARCH_KEYWORDS];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AssetSortKey {
    arch_rank: u8,
    suffix_rank: usize,
    name: String,
}

pub(crate) fn platform_asset_preferences(
    os: &str,
    arch: &str,
) -> (&'static [&'static str], &'static [&'static str]) {
    let target_suffixes = match os {
        "windows" => WINDOWS_SUFFIXES,
        "macos" => MACOS_SUFFIXES,
        _ => LINUX_SUFFIXES,
    };

    let arch_keywords = match arch {
        "x86_64" | "x64" | "amd64" => X64_ARCH_KEYWORDS,
        "aarch64" | "arm64" | "arm" => ARM64_ARCH_KEYWORDS,
        _ => EMPTY_ARCH_KEYWORDS,
    };

    (target_suffixes, arch_keywords)
}

pub(crate) fn select_best_asset_by_name<'a, T, F>(
    assets: &'a [T],
    name_of: F,
    target_suffixes: &[&str],
    arch_keywords: &[&str],
) -> Option<&'a T>
where
    F: Fn(&T) -> &str,
{
    assets
        .iter()
        .filter_map(|asset| {
            let key = asset_sort_key(name_of(asset), target_suffixes, arch_keywords)?;
            Some((key, asset))
        })
        .min_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, asset)| asset)
}

fn asset_sort_key(
    name: &str,
    target_suffixes: &[&str],
    arch_keywords: &[&str],
) -> Option<AssetSortKey> {
    let normalized_name = name.to_ascii_lowercase();
    let suffix_rank = target_suffixes
        .iter()
        .position(|suffix| normalized_name.ends_with(suffix))?;

    Some(AssetSortKey {
        arch_rank: asset_arch_rank(&normalized_name, arch_keywords),
        suffix_rank,
        name: normalized_name,
    })
}

fn asset_arch_rank(normalized_name: &str, arch_keywords: &[&str]) -> u8 {
    if arch_keywords
        .iter()
        .any(|keyword| normalized_name.contains(keyword))
    {
        return 0;
    }

    if KNOWN_ARCH_GROUPS.iter().any(|group| {
        !same_arch_group(group, arch_keywords)
            && group
                .iter()
                .any(|keyword| normalized_name.contains(keyword))
    }) {
        return 2;
    }

    1
}

fn same_arch_group(left: &[&str], right: &[&str]) -> bool {
    left.len() == right.len() && left.iter().zip(right.iter()).all(|(l, r)| l == r)
}
