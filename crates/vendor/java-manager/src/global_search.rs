//! 跨平台全局 Java 搜索和增量目录索引。

use crate::search::{GlobalSearchDirectory, GlobalSearchIndex, GlobalSearchOptions, SearchReport};
use crate::{JavaError, JavaInfo};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const EXCLUDED_DIRECTORY_NAMES: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    "target",
    "proc",
    "sys",
    "dev",
    "run",
    "lost+found",
];

/// 使用快速策略扫描全局环境、常见安装目录和可用文件系统根目录。
pub fn global_search() -> SearchReport {
    global_search_with_index(None, GlobalSearchOptions::fast())
}

/// 使用不限制深度的策略执行完整全局扫描。
pub fn global_search_complete() -> SearchReport {
    global_search_with_index(None, GlobalSearchOptions::complete())
}

/// 使用调用方保存的目录索引执行全局增量扫描。
pub fn global_search_with_index(
    previous: Option<&GlobalSearchIndex>,
    options: GlobalSearchOptions,
) -> SearchReport {
    let roots = global_search_roots();
    let mut report = SearchReport::default();

    // Windows 上优先复用 Everything 的现有索引；BFS 仍会补齐未被索引的路径。
    #[cfg(target_os = "windows")]
    report.merge_without_source_failure(crate::search::deep_search_with_diagnostics());

    let mut breadth_first = breadth_first_search(&roots, previous, options);
    report.index = breadth_first.index.take();
    report.merge_with_source_failure(breadth_first);
    deduplicate_installations(&mut report.installations);
    report
}

/// 扫描所有环境变量中的路径值，并对可用目录执行 BFS 搜索。
pub fn environment_search() -> SearchReport {
    environment_search_with_diagnostics(GlobalSearchOptions::fast())
}

/// 扫描所有环境变量中的路径值，并对可用目录执行 BFS 搜索。
pub fn environment_search_with_diagnostics(options: GlobalSearchOptions) -> SearchReport {
    let roots = environment_roots();
    breadth_first_search(&roots, None, options)
}

fn breadth_first_search(
    roots: &[PathBuf],
    previous: Option<&GlobalSearchIndex>,
    options: GlobalSearchOptions,
) -> SearchReport {
    let previous = previous.filter(|previous| can_reuse_index(previous, options));
    let roots = normalize_roots(roots);
    let mut report = SearchReport::default();
    let mut index = GlobalSearchIndex {
        roots: roots.clone(),
        max_depth: options.max_depth,
        max_directories: options.max_directories,
        ..GlobalSearchIndex::default()
    };
    let mut candidate_keys = HashSet::new();
    let previous_directories = previous
        .map(|value| {
            value
                .directories
                .iter()
                .map(|directory| (path_key(&directory.path), directory))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();

    if let Some(previous) = previous {
        for candidate in &previous.candidates {
            if is_under_any_root(candidate, &roots) {
                collect_candidate(&mut report, &mut index, &mut candidate_keys, candidate.clone());
            }
        }
    }

    let mut queue = VecDeque::new();
    let mut queued = HashSet::new();
    let mut visited = HashSet::new();

    for root in &roots {
        enqueue_directory(&mut queue, &mut queued, root.clone(), 0);
    }

    if let Some(previous) = previous {
        for directory in &previous.directories {
            if is_under_any_root(&directory.path, &roots)
                && let Some(depth) = relative_depth(&directory.path, &roots)
            {
                enqueue_directory(&mut queue, &mut queued, directory.path.clone(), depth);
            }
        }
    }

    while let Some((directory, depth)) = queue.pop_front() {
        let directory_key = path_key(&directory);
        if !visited.insert(directory_key.clone()) {
            continue;
        }

        if index.directories.len() >= options.max_directories {
            index.truncated = true;
            report.add_source_error(
                JavaError::RuntimeError(format!(
                    "Global search stopped after {} directories",
                    options.max_directories
                )),
                true,
            );
            break;
        }

        let metadata = match fs::symlink_metadata(&directory) {
            Ok(metadata) if metadata.is_dir() => metadata,
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                report.add_source_error(
                    JavaError::IoError(std::io::Error::new(
                        error.kind(),
                        format!(
                            "Failed to inspect global search directory '{}': {error}",
                            directory.display()
                        ),
                    )),
                    false,
                );
                continue;
            }
        };

        let current_index = directory_index(&directory, &metadata);
        let unchanged = previous_directories
            .get(&directory_key)
            .is_some_and(|old| **old == current_index);
        index.directories.push(current_index);

        // 增量扫描仍会检查旧索引中的子目录元数据，但不重新枚举未变化目录。
        if unchanged {
            continue;
        }

        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                report.add_source_error(
                    JavaError::IoError(std::io::Error::new(
                        error.kind(),
                        format!(
                            "Failed to read global search directory '{}': {error}",
                            directory.display()
                        ),
                    )),
                    false,
                );
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    report.add_source_error(JavaError::IoError(error), false);
                    continue;
                }
            };
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    report.add_source_error(JavaError::IoError(error), false);
                    continue;
                }
            };

            if is_java_executable_name(&path)
                && (file_type.is_file() || (file_type.is_symlink() && path.is_file()))
            {
                collect_candidate(&mut report, &mut index, &mut candidate_keys, path);
                continue;
            }

            let is_directory = file_type.is_dir()
                || (options.follow_links && file_type.is_symlink() && path.is_dir());
            if is_directory
                && should_descend(&path)
                && options.max_depth.is_none_or(|max_depth| depth < max_depth)
            {
                enqueue_directory(&mut queue, &mut queued, path, depth + 1);
            }
        }
    }

    index
        .directories
        .sort_by_key(|directory| path_key(&directory.path));
    index.candidates.sort_by_key(|path| path_key(path));
    index
        .candidates
        .dedup_by(|left, right| path_key(left) == path_key(right));
    report.index = Some(index);
    report
}

fn collect_candidate(
    report: &mut SearchReport,
    index: &mut GlobalSearchIndex,
    candidate_keys: &mut HashSet<String>,
    path: PathBuf,
) {
    let input_key = path_key(&path);
    if candidate_keys.contains(&input_key) {
        return;
    }

    match JavaInfo::from_discovered_path(path.to_string_lossy().into_owned()) {
        Ok(info) => {
            let canonical_path = info.path.clone();
            let canonical_key = path_key(&canonical_path);
            let is_new = !candidate_keys.contains(&canonical_key);
            candidate_keys.insert(input_key);
            candidate_keys.insert(canonical_key);
            if is_new {
                index.candidates.push(canonical_path);
                report.installations.push(info);
            }
        }
        Err(error) => report.add_candidate_error(path, error),
    }
}

fn can_reuse_index(previous: &GlobalSearchIndex, options: GlobalSearchOptions) -> bool {
    let depth_is_covered = match (previous.max_depth, options.max_depth) {
        (None, _) => true,
        (Some(previous), Some(current)) => previous >= current,
        (Some(_), None) => false,
    };
    let directory_limit_is_covered =
        !previous.truncated || previous.max_directories >= options.max_directories;

    depth_is_covered && directory_limit_is_covered
}

fn directory_index(path: &Path, metadata: &fs::Metadata) -> GlobalSearchDirectory {
    let modified_ns = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos().min(u64::MAX as u128) as u64)
        .unwrap_or_default();

    GlobalSearchDirectory {
        path: path.to_path_buf(),
        modified_ns,
        len: metadata.len(),
    }
}

fn enqueue_directory(
    queue: &mut VecDeque<(PathBuf, usize)>,
    queued: &mut HashSet<String>,
    path: PathBuf,
    depth: usize,
) {
    if queued.insert(path_key(&path)) {
        queue.push_back((path, depth));
    }
}

fn normalize_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for root in roots {
        let Ok(root) = fs::canonicalize(root) else {
            continue;
        };
        if !root.is_dir() {
            continue;
        }
        if seen.insert(path_key(&root)) {
            normalized.push(root);
        }
    }

    normalized
}

fn is_under_any_root(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn relative_depth(path: &Path, roots: &[PathBuf]) -> Option<usize> {
    roots
        .iter()
        .filter_map(|root| path.strip_prefix(root).ok())
        .map(|relative| relative.components().count())
        .min()
}

fn path_key(path: &Path) -> String {
    let value = path.to_string_lossy().into_owned();
    #[cfg(target_os = "windows")]
    {
        value.to_ascii_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        value
    }
}

fn is_java_executable_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    #[cfg(target_os = "windows")]
    {
        name.eq_ignore_ascii_case("java.exe") || name.eq_ignore_ascii_case("javaw.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        name == "java"
    }
}

fn should_descend(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    !EXCLUDED_DIRECTORY_NAMES
        .iter()
        .any(|excluded| name.eq_ignore_ascii_case(excluded))
}

fn global_search_roots() -> Vec<PathBuf> {
    let mut roots = common_search_roots();
    roots.extend(environment_roots());
    roots
}

fn environment_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    for (_name, value) in env::vars_os() {
        add_environment_path(&mut roots, PathBuf::from(&value));
        for path in env::split_paths(&value) {
            add_environment_path(&mut roots, path);
        }
    }

    roots
}

fn add_environment_path(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() || !path.exists() {
        return;
    }

    if path.is_dir() {
        roots.push(path);
    } else if path.is_file()
        && let Some(parent) = path.parent()
    {
        roots.push(parent.to_path_buf());
    }
}

fn common_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let home = home_dir();

    if let Some(home) = &home {
        for relative in [
            ".jabba",
            ".sdkman/candidates/java",
            ".asdf/installs/java",
            ".mise/installs/java",
            ".local/share/mise/installs/java",
            ".jdks",
            ".java",
            ".minecraft/runtime",
        ] {
            roots.push(home.join(relative));
        }
    }

    #[cfg(target_os = "windows")]
    {
        for variable in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            if let Some(path) = env::var_os(variable) {
                roots.push(PathBuf::from(path));
            }
        }

        if let Some(scoop) = env::var_os("SCOOP") {
            roots.push(PathBuf::from(scoop));
        }
        if let Some(home) = &home {
            roots.push(home.join("scoop"));
            roots.push(home.join(".jabba"));
        }

        for drive in 'A'..='Z' {
            let root = PathBuf::from(format!("{drive}:\\"));
            if root.is_dir() {
                roots.push(root);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        roots.extend([
            PathBuf::from("/"),
            PathBuf::from("/usr/lib/jvm"),
            PathBuf::from("/usr/java"),
            PathBuf::from("/opt"),
            PathBuf::from("/usr/local"),
            PathBuf::from("/mnt"),
            PathBuf::from("/media"),
        ]);
    }

    #[cfg(target_os = "macos")]
    {
        roots.extend([
            PathBuf::from("/"),
            PathBuf::from("/Library/Java/JavaVirtualMachines"),
            PathBuf::from("/System/Library/Java/JavaVirtualMachines"),
            PathBuf::from("/usr/local/opt"),
            PathBuf::from("/usr/local/Cellar"),
            PathBuf::from("/opt/homebrew/opt"),
            PathBuf::from("/opt/homebrew/Cellar"),
            PathBuf::from("/Volumes"),
        ]);
    }

    roots
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn deduplicate_installations(installations: &mut Vec<JavaInfo>) {
    let mut seen = HashSet::new();
    installations.retain(|info| seen.insert(path_key(&info.path)));
}

#[cfg(test)]
mod tests {
    use super::{GlobalSearchOptions, breadth_first_search};
    use std::fs;

    #[test]
    fn breadth_first_search_finds_java_and_reuses_directory_index() {
        let root = tempfile::tempdir().expect("temporary root should be created");
        let java_home = root.path().join("jdk-21");
        let java_bin = java_home.join("bin");
        fs::create_dir_all(&java_bin).expect("java bin should be created");
        fs::write(
            java_home.join("release"),
            "IMPLEMENTOR=\"Test JDK\"\nJAVA_VERSION=\"21.0.1\"\nOS_ARCH=\"amd64\"\n",
        )
        .expect("release metadata should be written");

        let java = java_bin.join(if cfg!(windows) { "java.exe" } else { "java" });
        fs::write(&java, []).expect("java placeholder should be written");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&java)
                .expect("java metadata should exist")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&java, permissions).expect("java should be executable");
        }

        let shallow = breadth_first_search(
            &[root.path().to_path_buf()],
            None,
            GlobalSearchOptions {
                max_depth: Some(0),
                ..GlobalSearchOptions::complete()
            },
        );
        assert!(shallow.installations.is_empty());

        let first = breadth_first_search(
            &[root.path().to_path_buf()],
            shallow.index.as_ref(),
            GlobalSearchOptions::complete(),
        );
        assert_eq!(first.installations.len(), 1);
        let index = first
            .index
            .as_ref()
            .expect("first scan should produce an index");

        let second = breadth_first_search(
            &[root.path().to_path_buf()],
            Some(index),
            GlobalSearchOptions::complete(),
        );
        assert_eq!(second.installations.len(), 1);
        assert!(!second.errors.iter().any(|error| error.path.is_some()));
    }
}
