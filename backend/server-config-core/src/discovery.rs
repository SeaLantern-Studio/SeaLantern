use std::collections::HashSet;
use std::path::{Path, PathBuf};

use regex::RegexBuilder;

use crate::startup::canonical_server_dir;
use crate::types::{
    DiscoveredServerConfigFile, KnownServerConfigRole, ServerConfigContentMatch,
    ServerConfigDiscoveryOptions, ServerConfigFileKind, ServerConfigJsonMode,
    ServerConfigOwnership, ServerConfigSearchHit, ServerConfigSearchMode,
    ServerConfigSearchScope, ServerConfigSourceKind,
};

const CONFIG_DISCOVERY_MAX_DEPTH: usize = 4;
const CONFIG_DISCOVERY_SKIP_DIRS: &[&str] = &[
    ".git",
    ".idea",
    ".vscode",
    "cache",
    "crash-reports",
    "libraries",
    "logs",
    "node_modules",
    "target",
    "versions",
    "world",
    "world_nether",
    "world_the_end",
];

pub const SUPPORTED_SERVER_CONFIG_EXTENSIONS: &[&str] = &["properties", "toml", "yaml", "yml"];

#[derive(Debug, Clone)]
struct DiscoverySource {
    scan_root: PathBuf,
    source_kind: ServerConfigSourceKind,
    source_label: String,
    display_prefix: String,
    server_root: Option<PathBuf>,
}

pub fn discover_server_config_files(
    server_path: &str,
) -> Result<Vec<DiscoveredServerConfigFile>, String> {
    discover_server_config_files_with_options(server_path, &ServerConfigDiscoveryOptions::default())
}

pub fn discover_server_config_files_with_options(
    server_path: &str,
    options: &ServerConfigDiscoveryOptions,
) -> Result<Vec<DiscoveredServerConfigFile>, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    discover_server_config_files_in_dir_with_options(&canonical_server, options)
}

pub fn discover_server_config_files_in_dir(
    server_dir: &Path,
) -> Result<Vec<DiscoveredServerConfigFile>, String> {
    discover_server_config_files_in_dir_with_options(server_dir, &ServerConfigDiscoveryOptions::default())
}

pub fn discover_server_config_files_in_dir_with_options(
    server_dir: &Path,
    options: &ServerConfigDiscoveryOptions,
) -> Result<Vec<DiscoveredServerConfigFile>, String> {
    if !server_dir.is_dir() {
        return Err("服务器目录无效".to_string());
    }

    let canonical_server = std::fs::canonicalize(server_dir)
        .map_err(|e| format!("解析服务器目录失败: {}", e))?;
    let mut seen = HashSet::new();
    let mut discovered = Vec::new();

    let mut sources = vec![build_server_root_source(&canonical_server)];
    sources.extend(build_manual_sources(&canonical_server, options)?);

    for source in sources {
        if source.scan_root.is_dir() {
            scan_dir(&source, &source.scan_root, 0, options, &mut seen, &mut discovered)?;
        } else if source.scan_root.is_file() {
            if let Some(entry) = discovered_entry_from_path(&source, &source.scan_root, options) {
                if seen.insert(entry.locator.clone()) {
                    discovered.push(entry);
                }
            }
        }
    }

    discovered.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.relative_path.cmp(&right.relative_path))
            .then_with(|| left.locator.cmp(&right.locator))
    });
    Ok(discovered)
}

pub fn search_server_config_files(
    server_path: &str,
    query: &str,
    mode: ServerConfigSearchMode,
    scope: ServerConfigSearchScope,
    limit: Option<usize>,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    search_server_config_files_with_options(
        server_path,
        &ServerConfigDiscoveryOptions::default(),
        query,
        mode,
        scope,
        limit,
        case_sensitive,
    )
}

pub fn search_server_config_files_with_options(
    server_path: &str,
    options: &ServerConfigDiscoveryOptions,
    query: &str,
    mode: ServerConfigSearchMode,
    scope: ServerConfigSearchScope,
    limit: Option<usize>,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let discovered = discover_server_config_files_in_dir_with_options(&canonical_server, options)?;
    search_server_config_files_in_entries(
        &discovered,
        query,
        mode,
        scope,
        limit,
        case_sensitive,
    )
}

pub fn search_server_config_files_in_entries(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    mode: ServerConfigSearchMode,
    scope: ServerConfigSearchScope,
    limit: Option<usize>,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        let hits = discovered
            .iter()
            .take(limit.unwrap_or(discovered.len()))
            .map(|entry| ServerConfigSearchHit {
                locator: entry.locator.clone(),
                relative_path: entry.relative_path.clone(),
                file_name: entry.file_name.clone(),
                absolute_path: entry.absolute_path.clone(),
                source_kind: entry.source_kind.clone(),
                source_label: entry.source_label.clone(),
                server_relative_path: entry.server_relative_path.clone(),
                kind: entry.kind.clone(),
                known_role: entry.known_role.clone(),
                ownership: entry.ownership.clone(),
                priority: entry.priority,
                score: u32::MAX - entry.priority,
                reason: "list_all".to_string(),
                content_match: None,
            })
            .collect();
        return Ok(hits);
    }

    let mut hits = match scope {
        ServerConfigSearchScope::Path => search_path_hits(discovered, trimmed, mode, case_sensitive)?,
        ServerConfigSearchScope::Content => search_content_hits(discovered, trimmed, mode, case_sensitive)?,
        ServerConfigSearchScope::All => {
            let mut path_hits = search_path_hits(discovered, trimmed, mode.clone(), case_sensitive)?;
            let content_hits = search_content_hits(discovered, trimmed, mode, case_sensitive)?;
            merge_search_hits(&mut path_hits, content_hits);
            path_hits
        }
    };

    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.priority.cmp(&right.priority))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
            .then_with(|| left.locator.cmp(&right.locator))
    });

    if let Some(limit) = limit {
        hits.truncate(limit);
    }

    Ok(hits)
}

pub fn resolve_primary_server_properties_path(server_path: &str) -> Result<PathBuf, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let discovered = discover_server_config_files_in_dir(&canonical_server)?;
    Ok(discovered
        .iter()
        .find(|entry| entry.known_role == Some(KnownServerConfigRole::ServerProperties))
        .map(|entry| PathBuf::from(&entry.absolute_path))
        .unwrap_or_else(|| canonical_server.join("server.properties")))
}

pub fn resolve_primary_startup_config_path(
    server_path: &str,
) -> Result<Option<(KnownServerConfigRole, PathBuf)>, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let discovered = discover_server_config_files_in_dir(&canonical_server)?;
    Ok(discovered
        .iter()
        .find_map(|entry| match entry.known_role.clone() {
            Some(KnownServerConfigRole::StartupPrimary | KnownServerConfigRole::StartupLegacy) => {
                entry
                    .known_role
                    .clone()
                    .map(|role| (role, PathBuf::from(&entry.absolute_path)))
            }
            _ => None,
        }))
}

pub fn resolve_primary_port_config_path(server_dir: &Path) -> Result<Option<PathBuf>, String> {
    let discovered = discover_server_config_files_in_dir(server_dir)?;
    Ok(discovered.iter().find_map(|entry| match entry.known_role {
        Some(KnownServerConfigRole::Pumpkin | KnownServerConfigRole::ServerProperties) => {
            Some(PathBuf::from(&entry.absolute_path))
        }
        _ => None,
    }))
}

pub fn resolve_discovered_config_path(
    server_path: &str,
    relative_path: &str,
) -> Result<PathBuf, String> {
    resolve_discovered_config_path_with_options(
        server_path,
        relative_path,
        None,
        &ServerConfigDiscoveryOptions::default(),
    )
}

pub fn resolve_discovered_config_path_with_options(
    server_path: &str,
    relative_path: &str,
    locator: Option<&str>,
    options: &ServerConfigDiscoveryOptions,
) -> Result<PathBuf, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let discovered = discover_server_config_files_in_dir_with_options(&canonical_server, options)?;
    let entry = discovered
        .into_iter()
        .find(|entry| match locator {
            Some(locator) => entry.locator == locator,
            None => entry.relative_path == relative_path,
        })
        .ok_or_else(|| match locator {
            Some(locator) => format!("未找到配置文件 locator={}", locator),
            None => format!("未找到配置文件: {}", relative_path),
        })?;
    Ok(PathBuf::from(entry.absolute_path))
}

fn build_server_root_source(server_root: &Path) -> DiscoverySource {
    DiscoverySource {
        scan_root: server_root.to_path_buf(),
        source_kind: ServerConfigSourceKind::ServerRoot,
        source_label: server_root.to_string_lossy().to_string(),
        display_prefix: String::new(),
        server_root: Some(server_root.to_path_buf()),
    }
}

fn build_manual_sources(
    server_root: &Path,
    options: &ServerConfigDiscoveryOptions,
) -> Result<Vec<DiscoverySource>, String> {
    let mut sources = Vec::new();

    for dir in &options.manual_import_dirs {
        let canonical = std::fs::canonicalize(dir)
            .map_err(|e| format!("解析手动导入目录失败 path={} error={}", dir, e))?;
        if !canonical.is_dir() {
            return Err(format!("手动导入目录无效: {}", dir));
        }
        let label = canonical.to_string_lossy().to_string();
        let prefix = format!("[import:{}]", display_name_from_path(&canonical));
        sources.push(DiscoverySource {
            scan_root: canonical,
            source_kind: ServerConfigSourceKind::ManualRoot,
            source_label: label,
            display_prefix: prefix,
            server_root: Some(server_root.to_path_buf()),
        });
    }

    for file in &options.manual_import_files {
        let canonical = std::fs::canonicalize(file)
            .map_err(|e| format!("解析手动导入文件失败 path={} error={}", file, e))?;
        if !canonical.is_file() {
            return Err(format!("手动导入文件无效: {}", file));
        }
        let label = canonical.to_string_lossy().to_string();
        sources.push(DiscoverySource {
            scan_root: canonical,
            source_kind: ServerConfigSourceKind::ManualFile,
            source_label: label,
            display_prefix: "[file]".to_string(),
            server_root: Some(server_root.to_path_buf()),
        });
    }

    Ok(sources)
}

fn scan_dir(
    source: &DiscoverySource,
    current: &Path,
    depth: usize,
    options: &ServerConfigDiscoveryOptions,
    seen: &mut HashSet<String>,
    discovered: &mut Vec<DiscoveredServerConfigFile>,
) -> Result<(), String> {
    let entries =
        std::fs::read_dir(current).map_err(|e| format!("读取服务器配置目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取服务器配置目录失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if depth >= CONFIG_DISCOVERY_MAX_DEPTH {
                continue;
            }
            if should_skip_dir(&path) {
                continue;
            }
            scan_dir(source, &path, depth + 1, options, seen, discovered)?;
            continue;
        }

        let Some(discovered_entry) = discovered_entry_from_path(source, &path, options) else {
            continue;
        };

        if seen.insert(discovered_entry.locator.clone()) {
            discovered.push(discovered_entry);
        }
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| CONFIG_DISCOVERY_SKIP_DIRS.contains(&name))
}

fn discovered_entry_from_path(
    source: &DiscoverySource,
    path: &Path,
    options: &ServerConfigDiscoveryOptions,
) -> Option<DiscoveredServerConfigFile> {
    let absolute_path = std::fs::canonicalize(path).ok()?;
    let file_name = absolute_path.file_name()?.to_str()?.to_string();
    let extension = absolute_path.extension()?.to_str()?.to_ascii_lowercase();
    let relative_under_root = match source.source_kind {
        ServerConfigSourceKind::ManualFile => file_name.clone(),
        _ => normalize_relative_path(absolute_path.strip_prefix(&source.scan_root).ok()?)?,
    };

    let kind = match extension.as_str() {
        "properties" => ServerConfigFileKind::Properties,
        "toml" => ServerConfigFileKind::Toml,
        "yaml" | "yml" => ServerConfigFileKind::Yaml,
        "json" => ServerConfigFileKind::Json,
        _ => return None,
    };

    let server_relative_path = source
        .server_root
        .as_ref()
        .and_then(|root| absolute_path.strip_prefix(root).ok())
        .and_then(normalize_relative_path);
    let role_path = server_relative_path
        .as_deref()
        .unwrap_or(relative_under_root.as_str());

    let (known_role, ownership, priority) =
        known_role_ownership_and_priority(role_path, &file_name, &kind)?;
    if kind == ServerConfigFileKind::Json
        && !should_include_json_file(role_path, &file_name, &known_role, options)
    {
        return None;
    }

    let relative_path = build_display_relative_path(source, &relative_under_root, &file_name);
    let absolute_string = absolute_path.to_string_lossy().to_string();
    let locator = format!(
        "{}:{}",
        source_kind_key(&source.source_kind),
        absolute_string.replace('\\', "/")
    );

    Some(DiscoveredServerConfigFile {
        locator,
        relative_path,
        file_name,
        absolute_path: absolute_string,
        source_kind: source.source_kind.clone(),
        source_label: source.source_label.clone(),
        server_relative_path,
        kind,
        known_role,
        ownership,
        priority,
    })
}

fn should_include_json_file(
    relative_path: &str,
    file_name: &str,
    known_role: &Option<KnownServerConfigRole>,
    options: &ServerConfigDiscoveryOptions,
) -> bool {
    if matches!(known_role, Some(KnownServerConfigRole::StartupLegacy)) {
        return true;
    }

    match options.json_mode {
        ServerConfigJsonMode::Disabled => false,
        ServerConfigJsonMode::All => true,
        ServerConfigJsonMode::Filtered => {
            !looks_like_filtered_generated_json(relative_path)
                && !looks_like_filtered_generated_json(file_name)
        }
    }
}

fn looks_like_filtered_generated_json(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    let stem = normalized.strip_suffix(".json").unwrap_or(&normalized);
    looks_like_uuid(stem) || looks_like_hash_like(stem)
}

fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let is_dash = matches!(index, 8 | 13 | 18 | 23);
        if is_dash {
            if *byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn looks_like_hash_like(value: &str) -> bool {
    let compact: String = value.chars().filter(|ch| *ch != '-' && *ch != '_').collect();
    let len = compact.len();
    len >= 16 && compact.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn build_display_relative_path(
    source: &DiscoverySource,
    relative_under_root: &str,
    file_name: &str,
) -> String {
    match source.source_kind {
        ServerConfigSourceKind::ServerRoot => relative_under_root.to_string(),
        ServerConfigSourceKind::ManualRoot => format!("{}/{}", source.display_prefix, relative_under_root),
        ServerConfigSourceKind::ManualFile => format!("{}/{}", source.display_prefix, file_name),
    }
}

fn display_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("import")
        .to_string()
}

fn source_kind_key(kind: &ServerConfigSourceKind) -> &'static str {
    match kind {
        ServerConfigSourceKind::ServerRoot => "server_root",
        ServerConfigSourceKind::ManualRoot => "manual_root",
        ServerConfigSourceKind::ManualFile => "manual_file",
    }
}

fn known_role_ownership_and_priority(
    relative_path: &str,
    file_name: &str,
    kind: &ServerConfigFileKind,
) -> Option<(Option<KnownServerConfigRole>, ServerConfigOwnership, u32)> {
    let normalized = relative_path.to_ascii_lowercase();
    if normalized == "sealantern/config.toml" {
        return Some((
            Some(KnownServerConfigRole::StartupPrimary),
            ServerConfigOwnership::ServiceManaged,
            0,
        ));
    }
    if file_name == "SL.json" || normalized == "sl.json" {
        return Some((
            Some(KnownServerConfigRole::StartupLegacy),
            ServerConfigOwnership::ServiceManaged,
            1,
        ));
    }
    if normalized == "pumpkin.toml" {
        return Some((
            Some(KnownServerConfigRole::Pumpkin),
            ServerConfigOwnership::ServerManaged,
            2,
        ));
    }
    if file_name.eq_ignore_ascii_case("server.properties") {
        return Some((
            Some(KnownServerConfigRole::ServerProperties),
            ServerConfigOwnership::ServerManaged,
            3,
        ));
    }

    let priority = match kind {
        ServerConfigFileKind::Properties => 10,
        ServerConfigFileKind::Toml => 20,
        ServerConfigFileKind::Yaml => 30,
        ServerConfigFileKind::Json => {
            if file_name.eq_ignore_ascii_case("SL.json") {
                1
            } else {
                40
            }
        }
    };
    Some((None, ServerConfigOwnership::ThirdParty, priority))
}

fn normalize_relative_path(relative: &Path) -> Option<String> {
    let parts = relative
        .iter()
        .map(|part| part.to_str())
        .collect::<Option<Vec<_>>>()?;
    Some(parts.join("/"))
}

fn search_path_hits(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    mode: ServerConfigSearchMode,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    match mode {
        ServerConfigSearchMode::Keyword => Ok(search_path_by_keyword(discovered, query, case_sensitive)),
        ServerConfigSearchMode::Regex => search_path_by_regex(discovered, query, case_sensitive),
        ServerConfigSearchMode::Similarity => Ok(search_path_by_similarity(discovered, query, case_sensitive)),
    }
}

fn search_content_hits(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    mode: ServerConfigSearchMode,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    match mode {
        ServerConfigSearchMode::Keyword => Ok(search_content_by_keyword(discovered, query, case_sensitive)),
        ServerConfigSearchMode::Regex => search_content_by_regex(discovered, query, case_sensitive),
        ServerConfigSearchMode::Similarity => Ok(search_content_by_similarity(discovered, query, case_sensitive)),
    }
}

fn search_path_by_keyword(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    case_sensitive: bool,
) -> Vec<ServerConfigSearchHit> {
    let normalized_query = normalize_text(query, case_sensitive);
    discovered
        .iter()
        .filter_map(|entry| {
            let file_name = normalize_text(&entry.file_name, case_sensitive);
            let relative_path = normalize_text(&entry.relative_path, case_sensitive);
            let absolute_path = normalize_text(&entry.absolute_path, case_sensitive);

            let (score, reason) = if file_name == normalized_query {
                (1_000, "file_name_exact")
            } else if relative_path == normalized_query {
                (960, "relative_path_exact")
            } else if file_name.contains(&normalized_query) {
                (860, "file_name_contains")
            } else if relative_path.contains(&normalized_query) {
                (760, "relative_path_contains")
            } else if absolute_path.contains(&normalized_query) {
                (700, "absolute_path_contains")
            } else {
                return None;
            };

            Some(build_search_hit(entry, score, reason, None))
        })
        .collect()
}

fn search_path_by_regex(
    discovered: &[DiscoveredServerConfigFile],
    pattern: &str,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    let regex = RegexBuilder::new(pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .map_err(|e| format!("配置文件搜索正则无效: {}", e))?;

    Ok(discovered
        .iter()
        .filter_map(|entry| {
            if regex.is_match(&entry.file_name) {
                return Some(build_search_hit(entry, 920, "file_name_regex", None));
            }
            if regex.is_match(&entry.relative_path) {
                return Some(build_search_hit(entry, 820, "relative_path_regex", None));
            }
            if regex.is_match(&entry.absolute_path) {
                return Some(build_search_hit(entry, 780, "absolute_path_regex", None));
            }
            None
        })
        .collect())
}

fn search_path_by_similarity(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    case_sensitive: bool,
) -> Vec<ServerConfigSearchHit> {
    let normalized_query = normalize_text(query, case_sensitive);
    let query_tokens = tokenize(&normalized_query);

    discovered
        .iter()
        .filter_map(|entry| {
            let file_name = normalize_text(&entry.file_name, case_sensitive);
            let relative_path = normalize_text(&entry.relative_path, case_sensitive);
            let absolute_path = normalize_text(&entry.absolute_path, case_sensitive);

            let file_score = similarity_score(&file_name, &normalized_query, &query_tokens);
            let relative_score = similarity_score(&relative_path, &normalized_query, &query_tokens);
            let absolute_score = similarity_score(&absolute_path, &normalized_query, &query_tokens);

            let (score, reason) = if file_score >= relative_score && file_score >= absolute_score {
                (file_score, "file_name_similarity")
            } else if relative_score >= absolute_score {
                (relative_score, "relative_path_similarity")
            } else {
                (absolute_score, "absolute_path_similarity")
            };

            if score == 0 {
                return None;
            }

            Some(build_search_hit(entry, score, reason, None))
        })
        .collect()
}

fn search_content_by_keyword(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    case_sensitive: bool,
) -> Vec<ServerConfigSearchHit> {
    let normalized_query = normalize_text(query, case_sensitive);

    discovered
        .iter()
        .filter_map(|entry| {
            let content = read_searchable_text(Path::new(&entry.absolute_path))?;
            let mut best_hit: Option<(u32, usize, String, &'static str)> = None;

            for (index, line) in content.lines().enumerate() {
                let normalized_line = normalize_text(line, case_sensitive);
                let score = if normalized_line == normalized_query {
                    Some((930, "content_line_exact"))
                } else if normalized_line.contains(&normalized_query) {
                    Some((780, "content_line_contains"))
                } else {
                    None
                };

                let Some((score, reason)) = score else {
                    continue;
                };
                if best_hit
                    .as_ref()
                    .map(|current| score > current.0)
                    .unwrap_or(true)
                {
                    best_hit = Some((score, index + 1, truncate_line(line), reason));
                }
            }

            best_hit.map(|(score, line_number, line_text, reason)| {
                build_search_hit(
                    entry,
                    score,
                    reason,
                    Some(ServerConfigContentMatch { line_number, line_text }),
                )
            })
        })
        .collect()
}

fn search_content_by_regex(
    discovered: &[DiscoveredServerConfigFile],
    pattern: &str,
    case_sensitive: bool,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    let regex = RegexBuilder::new(pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .map_err(|e| format!("配置内容搜索正则无效: {}", e))?;

    Ok(discovered
        .iter()
        .filter_map(|entry| {
            let content = read_searchable_text(Path::new(&entry.absolute_path))?;
            for (index, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    return Some(build_search_hit(
                        entry,
                        840,
                        "content_line_regex",
                        Some(ServerConfigContentMatch {
                            line_number: index + 1,
                            line_text: truncate_line(line),
                        }),
                    ));
                }
            }
            None
        })
        .collect())
}

fn search_content_by_similarity(
    discovered: &[DiscoveredServerConfigFile],
    query: &str,
    case_sensitive: bool,
) -> Vec<ServerConfigSearchHit> {
    let normalized_query = normalize_text(query, case_sensitive);
    let query_tokens = tokenize(&normalized_query);

    discovered
        .iter()
        .filter_map(|entry| {
            let content = read_searchable_text(Path::new(&entry.absolute_path))?;
            let mut best_hit: Option<(u32, usize, String)> = None;

            for (index, line) in content.lines().enumerate() {
                let normalized_line = normalize_text(line, case_sensitive);
                let score = similarity_score(&normalized_line, &normalized_query, &query_tokens);
                if score == 0 {
                    continue;
                }
                if best_hit
                    .as_ref()
                    .map(|current| score > current.0)
                    .unwrap_or(true)
                {
                    best_hit = Some((score, index + 1, truncate_line(line)));
                }
            }

            best_hit.map(|(score, line_number, line_text)| {
                build_search_hit(
                    entry,
                    score,
                    "content_line_similarity",
                    Some(ServerConfigContentMatch { line_number, line_text }),
                )
            })
        })
        .collect()
}

fn merge_search_hits(base: &mut Vec<ServerConfigSearchHit>, incoming: Vec<ServerConfigSearchHit>) {
    for hit in incoming {
        if let Some(existing) = base
            .iter_mut()
            .find(|existing| existing.locator == hit.locator)
        {
            if hit.score > existing.score {
                *existing = hit;
            }
            continue;
        }
        base.push(hit);
    }
}

fn build_search_hit(
    entry: &DiscoveredServerConfigFile,
    score: u32,
    reason: &str,
    content_match: Option<ServerConfigContentMatch>,
) -> ServerConfigSearchHit {
    ServerConfigSearchHit {
        locator: entry.locator.clone(),
        relative_path: entry.relative_path.clone(),
        file_name: entry.file_name.clone(),
        absolute_path: entry.absolute_path.clone(),
        source_kind: entry.source_kind.clone(),
        source_label: entry.source_label.clone(),
        server_relative_path: entry.server_relative_path.clone(),
        kind: entry.kind.clone(),
        known_role: entry.known_role.clone(),
        ownership: entry.ownership.clone(),
        priority: entry.priority,
        score,
        reason: reason.to_string(),
        content_match,
    }
}

fn read_searchable_text(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn truncate_line(line: &str) -> String {
    const MAX_CHARS: usize = 180;
    let trimmed = line.trim();
    let collected: String = trimmed.chars().take(MAX_CHARS).collect();
    if trimmed.chars().count() > MAX_CHARS {
        format!("{}...", collected)
    } else {
        collected
    }
}

fn normalize_text(value: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        value.to_string()
    } else {
        value.to_ascii_lowercase()
    }
}

fn tokenize(value: &str) -> Vec<&str> {
    value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect()
}

fn similarity_score(candidate: &str, query: &str, query_tokens: &[&str]) -> u32 {
    if candidate == query {
        return 980;
    }
    if candidate.starts_with(query) {
        return 900;
    }
    if candidate.contains(query) {
        return 820;
    }

    let candidate_tokens = tokenize(candidate);
    let overlap = query_tokens
        .iter()
        .filter(|token| candidate_tokens.iter().any(|candidate| candidate.contains(**token)))
        .count() as u32;
    if overlap > 0 {
        return 500 + overlap * 60;
    }

    if is_subsequence(query, candidate) {
        return 360;
    }

    0
}

fn is_subsequence(query: &str, candidate: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let mut query_chars = query.chars();
    let mut expected = query_chars.next();
    for ch in candidate.chars() {
        if Some(ch) == expected {
            expected = query_chars.next();
            if expected.is_none() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{
        discover_server_config_files_in_dir, discover_server_config_files_in_dir_with_options,
        search_server_config_files_in_entries, SUPPORTED_SERVER_CONFIG_EXTENSIONS,
    };
    use crate::types::{
        KnownServerConfigRole, ServerConfigDiscoveryOptions, ServerConfigJsonMode,
        ServerConfigOwnership, ServerConfigSearchMode, ServerConfigSearchScope,
        ServerConfigSourceKind,
    };

    #[test]
    fn supported_extensions_are_stable() {
        assert_eq!(SUPPORTED_SERVER_CONFIG_EXTENSIONS, &["properties", "toml", "yaml", "yml"]);
    }

    #[test]
    fn discover_server_config_files_marks_known_roles_and_ownership() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("SeaLantern")).unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(dir.path().join("SeaLantern").join("config.toml"), "max_memory = 2048\n")
            .unwrap();
        std::fs::write(dir.path().join("SL.json"), "{}\n").unwrap();
        std::fs::write(dir.path().join("server.properties"), "server-port=25565\n").unwrap();
        std::fs::write(dir.path().join("config").join("paper.yml"), "motd: test\n").unwrap();

        let discovered = discover_server_config_files_in_dir(dir.path()).unwrap();

        assert_eq!(discovered[0].known_role, Some(KnownServerConfigRole::StartupPrimary));
        assert_eq!(discovered[0].ownership, ServerConfigOwnership::ServiceManaged);
        assert_eq!(discovered[1].known_role, Some(KnownServerConfigRole::StartupLegacy));
        assert_eq!(discovered[2].known_role, Some(KnownServerConfigRole::ServerProperties));
        assert_eq!(discovered[2].ownership, ServerConfigOwnership::ServerManaged);
    }

    #[test]
    fn discover_server_config_files_supports_manual_imports_and_json_filtering() {
        let dir = tempfile::tempdir().unwrap();
        let manual = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("SeaLantern")).unwrap();
        std::fs::write(dir.path().join("SeaLantern").join("config.toml"), "max_memory = 2048\n")
            .unwrap();
        std::fs::write(manual.path().join("custom.yml"), "motd: hi\n").unwrap();
        std::fs::write(
            manual.path().join("123e4567-e89b-12d3-a456-426614174000.json"),
            "{\"ignored\":true}\n",
        )
        .unwrap();

        let discovered = discover_server_config_files_in_dir_with_options(
            dir.path(),
            &ServerConfigDiscoveryOptions {
                manual_import_dirs: vec![manual.path().to_string_lossy().to_string()],
                manual_import_files: vec![],
                json_mode: ServerConfigJsonMode::Filtered,
            },
        )
        .unwrap();

        assert!(discovered.iter().any(|entry| {
            entry.source_kind == ServerConfigSourceKind::ManualRoot && entry.file_name == "custom.yml"
        }));
        assert!(!discovered.iter().any(|entry| entry.file_name.ends_with(".json") && entry.known_role.is_none()));
    }

    #[test]
    fn search_server_config_files_supports_content_scope() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(
            dir.path().join("config").join("paper-global.yml"),
            "motd: hello world\nview-distance: 12\n",
        )
        .unwrap();

        let discovered = discover_server_config_files_in_dir(dir.path()).unwrap();
        let hits = search_server_config_files_in_entries(
            &discovered,
            "hello world",
            ServerConfigSearchMode::Keyword,
            ServerConfigSearchScope::Content,
            None,
            false,
        )
        .unwrap();

        assert_eq!(hits[0].reason, "content_line_contains");
        assert_eq!(hits[0].content_match.as_ref().map(|item| item.line_number), Some(1));
    }
}
