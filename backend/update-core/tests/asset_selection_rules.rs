#![allow(dead_code)]

#[path = "../src/asset_selector.rs"]
mod asset_selector;

use asset_selector::{platform_asset_preferences, select_best_asset_by_name};

#[derive(Debug)]
struct NamedAsset {
    name: &'static str,
}

fn pick_asset<'a>(
    assets: &'a [NamedAsset],
    os: &str,
    arch: &str,
) -> Option<&'a NamedAsset> {
    let (suffixes, arch_keywords) = platform_asset_preferences(os, arch);
    select_best_asset_by_name(assets, |asset| asset.name, suffixes, arch_keywords)
}

#[test]
fn windows_prefers_matching_arch_within_same_suffix_group() {
    let assets = [
        NamedAsset { name: "Sea.Lantern_1.0.0_windows_arm64_setup.msi" },
        NamedAsset { name: "Sea.Lantern_1.0.0_windows_x64_setup.msi" },
    ];

    let selected = pick_asset(&assets, "windows", "x86_64")
        .expect("matching windows asset should be selected");

    assert_eq!(selected.name, "Sea.Lantern_1.0.0_windows_x64_setup.msi");
}

#[test]
fn windows_prefers_unknown_arch_over_explicit_wrong_arch_when_no_match_exists() {
    let assets = [
        NamedAsset { name: "Sea.Lantern_1.0.0_windows_arm64_setup.msi" },
        NamedAsset { name: "Sea.Lantern_1.0.0_windows_setup.msi" },
    ];

    let selected = pick_asset(&assets, "windows", "x86_64")
        .expect("fallback windows asset should be selected");

    assert_eq!(selected.name, "Sea.Lantern_1.0.0_windows_setup.msi");
}

#[test]
fn windows_prefers_matching_arch_over_wrong_arch_even_when_suffix_rank_is_lower() {
    let assets = [
        NamedAsset { name: "Sea.Lantern_1.0.0_windows_x64_portable.exe" },
        NamedAsset { name: "Sea.Lantern_1.0.0_windows_arm64_setup.msi" },
    ];

    let selected = pick_asset(&assets, "windows", "x86_64")
        .expect("matching windows asset should be selected");

    assert_eq!(selected.name, "Sea.Lantern_1.0.0_windows_x64_portable.exe");
}

#[test]
fn macos_prefers_matching_arch_dmg_over_other_arch_dmg() {
    let assets = [
        NamedAsset { name: "Sea.Lantern_1.0.0_aarch64.dmg" },
        NamedAsset { name: "Sea.Lantern_1.0.0_x64.dmg" },
    ];

    let selected = pick_asset(&assets, "macos", "x86_64")
        .expect("matching macos asset should be selected");

    assert_eq!(selected.name, "Sea.Lantern_1.0.0_x64.dmg");
}

#[test]
fn linux_prefers_matching_arch_appimage() {
    let assets = [
        NamedAsset { name: "Sea.Lantern_1.0.0_linux_arm64.AppImage" },
        NamedAsset { name: "Sea.Lantern_1.0.0_linux_x64.AppImage" },
    ];

    let selected = pick_asset(&assets, "linux", "x86_64")
        .expect("matching linux asset should be selected");

    assert_eq!(selected.name, "Sea.Lantern_1.0.0_linux_x64.AppImage");
}
