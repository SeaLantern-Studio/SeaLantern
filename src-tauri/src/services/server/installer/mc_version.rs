use std::collections::HashMap;
use std::path::Path;

use crate::utils::constants::STARTER_MC_VERSION_OPTIONS;

pub fn detect_mc_version_from_mods(root_dir: &Path) -> (Option<String>, bool) {
    let mods_dir = root_dir.join("mods");
    if !mods_dir.exists() || !mods_dir.is_dir() {
        return (None, true);
    }

    let mut filenames = Vec::new();
    collect_mod_filenames(&mods_dir, &mut filenames);
    if filenames.is_empty() {
        return (None, true);
    }

    let mut version_counter: HashMap<&'static str, usize> = HashMap::new();
    for filename in &filenames {
        let lowered = filename.to_ascii_lowercase();
        for version in STARTER_MC_VERSION_OPTIONS {
            let version_lower = version.to_ascii_lowercase();
            if contains_mc_version_token(&lowered, &version_lower) {
                *version_counter.entry(version).or_insert(0) += 1;
            }
        }
    }

    let max_count = version_counter.values().copied().max().unwrap_or(0);
    if max_count == 0 {
        return (None, true);
    }

    let mut winners = version_counter
        .iter()
        .filter_map(|(version, count)| {
            if *count == max_count {
                Some((*version).to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();
    winners.sort();

    if winners.len() != 1 {
        return (None, true);
    }

    (winners.first().cloned(), false)
}

fn collect_mod_filenames(dir: &Path, output: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_mod_filenames(&path, output);
            continue;
        }

        if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
            output.push(filename.to_string());
        }
    }
}

fn contains_mc_version_token(filename: &str, version: &str) -> bool {
    let mut search_from = 0usize;
    while let Some(index) = filename[search_from..].find(version) {
        let absolute_index = search_from + index;
        let previous_char = filename[..absolute_index].chars().last();
        if previous_char.map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
            search_from = absolute_index + 1;
            continue;
        }

        let end_index = absolute_index + version.len();
        let suffix = &filename[end_index..];
        let next_char = suffix.chars().next();
        if let Some(ch) = next_char {
            if ch.is_ascii_digit() {
                search_from = end_index;
                continue;
            }

            if ch == '.' {
                let second = suffix.chars().nth(1);
                if second.map(|value| value.is_ascii_digit()).unwrap_or(false) {
                    search_from = end_index;
                    continue;
                }
            }
        }

        return true;
    }

    false
}
