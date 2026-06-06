use std::collections::HashMap;
use std::path::Path;

pub fn detect_mc_version_from_mods(
    root_dir: &Path,
    known_versions: &[&str],
) -> (Option<String>, bool) {
    detect_mc_version_from_mods_checked(root_dir, known_versions).unwrap_or((None, true))
}

pub fn detect_mc_version_from_mods_checked(
    root_dir: &Path,
    known_versions: &[&str],
) -> Result<(Option<String>, bool), String> {
    let mods_dir = root_dir.join("mods");
    if !mods_dir.exists() || !mods_dir.is_dir() {
        return Ok((None, true));
    }

    let mut filenames = Vec::new();
    collect_mod_filenames_checked(&mods_dir, &mut filenames)?;
    if filenames.is_empty() {
        return Ok((None, true));
    }

    let mut version_counter: HashMap<&str, usize> = HashMap::new();
    for filename in &filenames {
        let lowered = filename.to_ascii_lowercase();
        for version in known_versions {
            let version_lower = version.to_ascii_lowercase();
            if contains_mc_version_token(&lowered, &version_lower) {
                *version_counter.entry(*version).or_insert(0) += 1;
            }
        }
    }

    let max_count = version_counter.values().copied().max().unwrap_or(0);
    if max_count == 0 {
        return Ok((None, true));
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
        return Ok((None, true));
    }

    Ok((winners.first().cloned(), false))
}

fn collect_mod_filenames_checked(dir: &Path, output: &mut Vec<String>) -> Result<(), String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            return Err(format!("读取 mods 目录失败: {}", error));
        }
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取 mods 目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
            {
                return Err(format!(
                    "检测到目录伪装成 mod JAR 文件: {}",
                    path.to_string_lossy()
                ));
            }
            collect_mod_filenames_checked(&path, output)?;
            continue;
        }

        if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
            output.push(filename.to_string());
        }
    }

    Ok(())
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
