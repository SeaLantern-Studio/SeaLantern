use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{command, AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub latest_version: String,
    pub current_version: String,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub source: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

struct RepoConfig {
    owner: &'static str,
    repo: &'static str,
    api_base: &'static str,
}

static INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

impl RepoConfig {
    fn api_url(&self) -> String {
        format!("{}/{}/{}/releases/latest", self.api_base, self.owner, self.repo)
    }
}

fn get_github_config() -> RepoConfig {
    RepoConfig {
        owner: "FPSZ",
        repo: "SeaLantern",
        api_base: "https://api.github.com/repos",
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    body: Option<String>,
    assets: Vec<ReleaseAsset>,
    published_at: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let config = get_github_config();
    fetch_release(&client, &config, current_version).await
}

async fn fetch_release(
    client: &reqwest::Client,
    config: &RepoConfig,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    let url = config.api_url();

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API status: {}", resp.status()));
    }

    let release: ReleaseResponse = resp
        .json()
        .await
        .map_err(|e| format!("response parse failed: {}", e))?;

    let latest_version = normalize_release_tag_version(&release.tag_name);
    let is_newer_version = compare_versions(current_version, &latest_version);
    let selected_asset = find_suitable_asset(&release.assets);
    let download_url = selected_asset.map(|asset| asset.browser_download_url.clone());
    let sha256 = if is_newer_version {
        if let Some(asset) = selected_asset {
            resolve_asset_sha256(client, &release.assets, asset).await
        } else {
            None
        }
    } else {
        None
    };

    let has_update = is_newer_version && download_url.is_some();

    Ok(UpdateInfo {
        has_update,
        latest_version,
        current_version: current_version.to_string(),
        download_url,
        release_notes: release.body,
        published_at: release.published_at.or(release.created_at),
        source: Some("github".to_string()),
        sha256,
    })
}

fn find_suitable_asset(assets: &[ReleaseAsset]) -> Option<&ReleaseAsset> {
    let target_suffixes: &[&str] = if cfg!(target_os = "windows") {
        &[".msi", ".exe"]
    } else if cfg!(target_os = "macos") {
        &[".dmg", ".app", ".tar.gz"]
    } else {
        &[".appimage", ".deb", ".rpm", ".tar.gz"]
    };

    for suffix in target_suffixes {
        if let Some(asset) = assets.iter().find(|a| {
            let name = a.name.to_ascii_lowercase();
            name.ends_with(suffix)
        }) {
            return Some(asset);
        }
    }

    None
}

async fn resolve_asset_sha256(
    client: &reqwest::Client,
    assets: &[ReleaseAsset],
    target_asset: &ReleaseAsset,
) -> Option<String> {
    let candidates = find_sha256_assets(assets, &target_asset.name);
    for hash_asset in candidates {
        if let Some(hash) = fetch_sha256_from_asset(client, hash_asset, &target_asset.name).await {
            return Some(hash);
        }
    }
    None
}

fn find_sha256_assets<'a>(assets: &'a [ReleaseAsset], target_name: &str) -> Vec<&'a ReleaseAsset> {
    let target_lower = target_name.to_ascii_lowercase();
    let target_file_name = Path::new(target_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(target_name)
        .to_ascii_lowercase();

    let exact_names = [
        format!("{target_lower}.sha256"),
        format!("{target_lower}.sha256sum"),
        format!("{target_lower}.sha256.txt"),
        format!("{target_lower}.sha256sums"),
    ];

    let mut primary = Vec::new();
    let mut secondary = Vec::new();
    let mut generic = Vec::new();

    for asset in assets {
        let name = asset.name.to_ascii_lowercase();
        if exact_names.iter().any(|item| item == &name) {
            primary.push(asset);
            continue;
        }

        let is_hash_file =
            name.contains("sha256") || name.contains("checksum") || name.contains("checksums");
        if !is_hash_file {
            continue;
        }

        if name.contains(&target_lower) {
            primary.push(asset);
            continue;
        }

        if name.contains(&target_file_name) {
            secondary.push(asset);
        } else {
            generic.push(asset);
        }
    }

    primary.extend(secondary);
    primary.extend(generic);
    primary
}

async fn fetch_sha256_from_asset(
    client: &reqwest::Client,
    hash_asset: &ReleaseAsset,
    target_name: &str,
) -> Option<String> {
    let response = client
        .get(&hash_asset.browser_download_url)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    if let Some(content_length) = response.content_length() {
        if content_length > 1024 * 1024 {
            return None;
        }
    }

    let content = response.text().await.ok()?;
    parse_sha256_from_checksum_content(&content, target_name)
}

fn parse_sha256_from_checksum_content(content: &str, target_name: &str) -> Option<String> {
    let target_lower = target_name.to_ascii_lowercase();
    let target_file_name = Path::new(target_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(target_name)
        .to_ascii_lowercase();

    let mut single_hash: Option<String> = None;
    let mut hash_line_count = 0_usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let hash = match find_sha256_in_line(trimmed) {
            Some(value) => value,
            None => continue,
        };

        hash_line_count += 1;
        if hash_line_count == 1 {
            single_hash = Some(hash.clone());
        } else {
            single_hash = None;
        }

        let line_lower = trimmed.to_ascii_lowercase();
        if line_lower.contains(&target_lower) || line_lower.contains(&target_file_name) {
            return Some(hash);
        }
    }

    if hash_line_count == 1 {
        return single_hash;
    }

    None
}

fn find_sha256_in_line(line: &str) -> Option<String> {
    for token in line.split(|ch: char| {
        ch.is_ascii_whitespace()
            || matches!(ch, '=' | ':' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>')
    }) {
        let candidate = token.trim_matches(|ch| ch == '*' || ch == '"' || ch == '\'');
        if is_sha256_hex(candidate) {
            return Some(candidate.to_ascii_lowercase());
        }
    }

    None
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let current_v = parse_version(current);
    let latest_v = parse_version(latest);
    latest_v > current_v
}

fn normalize_release_tag_version(tag_name: &str) -> String {
    let trimmed = tag_name.trim();
    if let Some(extracted) = extract_semver_from_text(trimmed) {
        return extracted;
    }

    trimmed.trim_start_matches(['v', 'V']).to_string()
}

fn extract_semver_from_text(input: &str) -> Option<String> {
    let regex =
        Regex::new(r"(?i)\bv?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)\b").ok()?;

    let mut last_match: Option<String> = None;
    for captures in regex.captures_iter(input) {
        if let Some(matched) = captures.get(1) {
            last_match = Some(matched.as_str().to_string());
        }
    }

    last_match
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParsedVersion {
    core: [u64; 3],
    pre: Option<Vec<PreIdent>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum PreIdent {
    Numeric(u64),
    AlphaNum(String),
}

impl Ord for PreIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            (Self::Numeric(a), Self::Numeric(b)) => a.cmp(b),
            (Self::Numeric(_), Self::AlphaNum(_)) => Ordering::Less,
            (Self::AlphaNum(_), Self::Numeric(_)) => Ordering::Greater,
            (Self::AlphaNum(a), Self::AlphaNum(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for PreIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParsedVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match self.core.cmp(&other.core) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => {
                for i in 0..std::cmp::max(a.len(), b.len()) {
                    match (a.get(i), b.get(i)) {
                        (Some(x), Some(y)) => match x.cmp(y) {
                            Ordering::Equal => continue,
                            ord => return ord,
                        },
                        (Some(_), None) => return Ordering::Greater,
                        (None, Some(_)) => return Ordering::Less,
                        (None, None) => return Ordering::Equal,
                    }
                }

                Ordering::Equal
            }
        }
    }
}

impl PartialOrd for ParsedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_version(input: &str) -> ParsedVersion {
    let normalized = input.trim().trim_start_matches(['v', 'V']);
    let no_build = normalized.split('+').next().unwrap_or(normalized);

    let (core_part, pre_part) = no_build
        .split_once('-')
        .map_or((no_build, None), |(core, pre)| (core, Some(pre)));

    let mut core = [0_u64; 3];
    for (idx, piece) in core_part.split('.').take(3).enumerate() {
        core[idx] = piece.trim().parse::<u64>().unwrap_or(0);
    }

    let pre = pre_part.and_then(|p| {
        let idents: Vec<PreIdent> = p
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| match s.parse::<u64>() {
                Ok(n) => PreIdent::Numeric(n),
                Err(_) => PreIdent::AlphaNum(s.to_ascii_lowercase()),
            })
            .collect();

        if idents.is_empty() {
            None
        } else {
            Some(idents)
        }
    });

    ParsedVersion { core, pre }
}

#[command]
pub async fn open_download_url(url: String) -> Result<(), String> {
    opener::open(&url).map_err(|e| format!("open link failed: {}", e))
}

fn get_update_cache_dir() -> PathBuf {
    let cache_dir = dirs_next::cache_dir().unwrap_or_else(std::env::temp_dir);
    cache_dir.join("com.fpsz.sea-lantern").join("updates")
}

fn get_pending_update_file() -> PathBuf {
    get_update_cache_dir().join("pending_update.json")
}

fn file_name_from_url(url: &str) -> String {
    let candidate = url.rsplit('/').next().unwrap_or("update");
    let candidate = candidate.split('?').next().unwrap_or("update");
    let candidate = candidate.split('#').next().unwrap_or("update");
    if candidate.trim().is_empty() {
        "update".to_string()
    } else {
        candidate.to_string()
    }
}

#[cfg(target_os = "windows")]
fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(target_os = "windows")]
fn build_hidden_powershell_command(command: &str) -> std::process::Command {
    let mut process = std::process::Command::new("powershell");
    process.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-WindowStyle",
        "Hidden",
        "-Command",
        command,
    ]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        process.creation_flags(CREATE_NO_WINDOW);
    }

    process
}

#[cfg(target_os = "windows")]
fn spawn_update_relaunch_watcher(
    installer_pid: u32,
    relaunch_exe: &str,
    cleanup_file_path: Option<&str>,
    pending_file_path: Option<&str>,
) -> Result<(), String> {
    let relaunch_exe_escaped = escape_powershell_single_quoted(relaunch_exe);
    let cleanup_file_script = cleanup_file_path
        .map(escape_powershell_single_quoted)
        .map(|path| {
            format!(
                "if (Test-Path '{path}') {{ Remove-Item -Path '{path}' -Force -ErrorAction SilentlyContinue }}; "
            )
        })
        .unwrap_or_default();
    let cleanup_pending_script = pending_file_path
        .map(escape_powershell_single_quoted)
        .map(|path| {
            format!(
                "if (Test-Path '{path}') {{ Remove-Item -Path '{path}' -Force -ErrorAction SilentlyContinue }}; "
            )
        })
        .unwrap_or_default();
    let watcher_command = format!(
        "$ErrorActionPreference = 'SilentlyContinue'; \
         try {{ \
           $installer = [System.Diagnostics.Process]::GetProcessById({installer_pid}); \
           if ($installer) {{ \
             $installer.WaitForExit(); \
             if ($installer.ExitCode -eq 0) {{ \
               {cleanup_file_script}\
               {cleanup_pending_script}\
               Start-Sleep -Milliseconds 700; \
               Start-Process -FilePath '{relaunch_exe_escaped}' \
             }} \
           }} \
         }} catch {{}}"
    );

    build_hidden_powershell_command(&watcher_command)
        .spawn()
        .map_err(|e| format!("Failed to watch installer process: {}", e))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn spawn_elevated_windows_process(
    file_path: &str,
    args: &[&str],
    cleanup_file_path: Option<&str>,
    pending_file_path: Option<&str>,
) -> Result<(), String> {
    let file_path_escaped = escape_powershell_single_quoted(file_path);
    let argument_list = args
        .iter()
        .map(|arg| format!("'{}'", escape_powershell_single_quoted(arg)))
        .collect::<Vec<String>>()
        .join(", ");

    let launch_command = format!(
        "$ErrorActionPreference = 'Stop'; \
         $installer = Start-Process -FilePath '{file_path_escaped}' -ArgumentList @({argument_list}) -Verb RunAs -PassThru; \
         if (-not $installer) {{ throw 'Installer process was not created.' }}; \
         Write-Output $installer.Id"
    );

    let output = build_hidden_powershell_command(&launch_command)
        .output()
        .map_err(|e| format!("Failed to request administrator privileges: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Administrator permission was denied or installer failed to launch".to_string()
        } else {
            format!("Administrator permission was denied or installer failed to launch: {}", stderr)
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let installer_pid = stdout
        .lines()
        .rev()
        .find_map(|line| line.trim().parse::<u32>().ok())
        .ok_or_else(|| "Installer started but process id was not returned".to_string())?;

    if let Some(relaunch_exe) = std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(|value| value.to_string()))
    {
        spawn_update_relaunch_watcher(
            installer_pid,
            &relaunch_exe,
            cleanup_file_path,
            pending_file_path,
        )?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingUpdate {
    pub file_path: String,
    pub version: String,
}

#[command]
pub async fn download_update(
    app: AppHandle,
    url: String,
    expected_hash: Option<String>,
) -> Result<String, String> {
    let cache_dir = get_update_cache_dir();
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let file_name = file_name_from_url(&url);
    let file_path = cache_dir.join(file_name);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0_u64;

    let mut file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            "update-download-progress",
            DownloadProgress { downloaded, total: total_size, percent },
        );
    }

    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    let file_path_str = file_path.to_string_lossy().to_string();

    if let Some(hash) = expected_hash {
        let calculated_hash =
            calculate_sha256(&file_path).map_err(|e| format!("Failed to calculate hash: {}", e))?;

        if calculated_hash.to_lowercase() != hash.to_lowercase() {
            std::fs::remove_file(&file_path).ok();
            return Err(format!(
                "Hash verification failed. Expected: {}, Got: {}",
                hash, calculated_hash
            ));
        }
    }

    Ok(file_path_str)
}

fn calculate_sha256(file_path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[command]
pub async fn install_update(file_path: String, version: String) -> Result<(), String> {
    if INSTALL_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return Err("Install is already in progress".to_string());
    }

    let result = (|| -> Result<(), String> {
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(format!("Update file not found: {}", file_path));
        }

        let pending_file = get_pending_update_file();
        if let Some(parent) = pending_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create pending update directory: {}", e))?;
        }

        let pending = PendingUpdate { file_path: file_path.clone(), version };

        let json = serde_json::to_string(&pending)
            .map_err(|e| format!("Failed to serialize pending update: {}", e))?;

        std::fs::write(&pending_file, json)
            .map_err(|e| format!("Failed to write pending update file: {}", e))?;

        #[cfg(target_os = "windows")]
        {
            let pending_file_path = pending_file.to_string_lossy().to_string();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            match extension.as_deref() {
                Some("msi") => {
                    spawn_elevated_windows_process(
                        "msiexec.exe",
                        &["/i", &file_path, "/passive", "/norestart"],
                        Some(&file_path),
                        Some(pending_file_path.as_str()),
                    )?;
                }
                Some("exe") => {
                    spawn_elevated_windows_process(
                        &file_path,
                        &["/S", "/norestart"],
                        Some(&file_path),
                        Some(pending_file_path.as_str()),
                    )?;
                }
                _ => {
                    opener::open(&file_path)
                        .map_err(|e| format!("Failed to open update file: {}", e))?;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            opener::open(&file_path).map_err(|e| format!("Failed to open update file: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            opener::open(&file_path).map_err(|e| format!("Failed to open update file: {}", e))?;
        }

        Ok(())
    })();

    if result.is_err() {
        INSTALL_IN_PROGRESS.store(false, Ordering::SeqCst);
        std::fs::remove_file(get_pending_update_file()).ok();
    }

    result
}

#[command]
pub async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    let pending_file = get_pending_update_file();

    if !pending_file.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&pending_file)
        .map_err(|e| format!("Failed to read pending update file: {}", e))?;

    let pending: PendingUpdate = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse pending update: {}", e))?;

    let path = PathBuf::from(&pending.file_path);
    if !path.exists() {
        std::fs::remove_file(&pending_file).ok();
        return Ok(None);
    }

    let current_version = env!("CARGO_PKG_VERSION");
    if !compare_versions(current_version, &pending.version) {
        std::fs::remove_file(&pending_file).ok();
        return Ok(None);
    }

    Ok(Some(pending))
}

#[command]
pub async fn clear_pending_update() -> Result<(), String> {
    let pending_file = get_pending_update_file();
    if pending_file.exists() {
        std::fs::remove_file(&pending_file)
            .map_err(|e| format!("Failed to remove pending update file: {}", e))?;
    }
    Ok(())
}

#[command]
pub async fn restart_and_install(app: AppHandle) -> Result<(), String> {
    app.restart();
}

#[command]
pub async fn download_update_from_debug_url(app: AppHandle, url: String) -> Result<String, String> {
    download_update(app, url, None).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_handles_prerelease() {
        assert!(compare_versions("1.2.3-beta.1", "1.2.3"));
        assert!(!compare_versions("1.2.3", "1.2.3-beta.1"));
        assert!(compare_versions("1.2.3-beta.1", "1.2.3-beta.2"));
        assert!(!compare_versions("1.2.3-rc.2", "1.2.3-rc.1"));
    }

    #[test]
    fn compare_versions_handles_basic_semver() {
        assert!(compare_versions("1.2.3", "1.2.4"));
        assert!(!compare_versions("1.2.4", "1.2.3"));
        assert!(compare_versions("v1.9.9", "2.0.0"));
        assert!(!compare_versions("2.0.0", "2.0.0"));
    }

    #[test]
    fn parse_version_ignores_build_metadata() {
        assert_eq!(parse_version("1.2.3+abc"), parse_version("1.2.3+def"));
    }

    #[test]
    fn parse_sha256_matches_target_file() {
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let content = format!("{hash}  SeaLantern-setup.exe");
        assert_eq!(
            parse_sha256_from_checksum_content(&content, "SeaLantern-setup.exe"),
            Some(hash.to_string())
        );
    }

    #[test]
    fn parse_sha256_accepts_single_hash_file() {
        let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(
            parse_sha256_from_checksum_content(hash, "SeaLantern-setup.exe"),
            Some(hash.to_string())
        );
    }

    #[test]
    fn parse_sha256_rejects_multi_hash_without_target_match() {
        let first = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let second = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
        let content = format!("{first} other.exe\n{second} another.exe");
        assert_eq!(parse_sha256_from_checksum_content(&content, "SeaLantern-setup.exe"), None);
    }

    #[test]
    fn normalize_release_tag_version_handles_prefixed_tag() {
        assert_eq!(normalize_release_tag_version("sea-lantern-v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_plain_version_tag() {
        assert_eq!(normalize_release_tag_version("v0.5.0"), "0.5.0");
    }

    #[test]
    fn normalize_release_tag_version_handles_prerelease_tag() {
        assert_eq!(normalize_release_tag_version("SeaLantern_release-v1.2.3-rc.1"), "1.2.3-rc.1");
    }
}
