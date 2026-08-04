use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::archive::{self, ArchiveMetadata};
use super::error::ServerInspectionError;
use super::model::{DiagnosticSeverity, InspectionDiagnostic};
use super::InspectionOptions;
use crate::provisioning::StartupScriptKind;

const ROOT_SCRIPT_NAMES: &[&str] =
    &["run.bat", "run.sh", "run.ps1", "start.bat", "start.sh", "start.ps1"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModLoaderFamily {
    Forge,
    NeoForge,
}

pub(super) struct MetadataFile {
    pub(super) relative_path: PathBuf,
    pub(super) content: Vec<u8>,
}

pub(super) struct RootArchive {
    pub(super) relative_path: PathBuf,
    pub(super) metadata: ArchiveMetadata,
}

pub(super) struct ModLoaderInstallation {
    pub(super) family: ModLoaderFamily,
    pub(super) coordinate_version: String,
    pub(super) relative_directory: PathBuf,
    pub(super) windows_args: Option<MetadataFile>,
    pub(super) unix_args: Option<MetadataFile>,
    pub(super) nested_archive_path: Option<PathBuf>,
    pub(super) nested_metadata: Option<ArchiveMetadata>,
}

pub(super) struct StartupScript {
    pub(super) relative_path: PathBuf,
    pub(super) kind: StartupScriptKind,
    pub(super) content: String,
}

pub(super) struct DirectoryMetadata {
    pub(super) root_archives: Vec<RootArchive>,
    pub(super) installations: Vec<ModLoaderInstallation>,
    pub(super) scripts: Vec<StartupScript>,
    pub(super) diagnostics: Vec<InspectionDiagnostic>,
}

pub(super) fn read_metadata(
    path: &Path,
    options: &InspectionOptions,
) -> Result<DirectoryMetadata, ServerInspectionError> {
    let mut consumed = 0_u64;
    let mut diagnostics = Vec::new();
    let root_entries = sorted_entries(path, options)?;
    let mut root_candidates = Vec::new();
    for entry in root_entries {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(source) => {
                diagnostics.push(limit_diagnostic(
                    "root_entry_unreadable",
                    format!(
                        "root entry {} could not be inspected and was skipped: {source}",
                        entry.path().display()
                    ),
                ));
                continue;
            }
        };
        if file_type.is_file() && is_root_jar(&entry.file_name().to_string_lossy()) {
            root_candidates.push(entry);
        }
    }
    root_candidates
        .sort_by_key(|entry| root_archive_sort_key(&entry.file_name().to_string_lossy()));
    let mut root_archives = Vec::new();
    if root_candidates.len() > options.max_root_archives {
        diagnostics.push(limit_diagnostic(
            "root_archive_limit_reached",
            format!(
                "directory {} contains {} root JARs; only the first {} were inspected",
                path.display(),
                root_candidates.len(),
                options.max_root_archives
            ),
        ));
        root_candidates.truncate(options.max_root_archives);
    }
    for entry in root_candidates {
        let relative_path = PathBuf::from(entry.file_name());
        match archive::read_metadata_with_budget(&entry.path(), options, &mut consumed) {
            Ok(mut metadata) => {
                diagnostics.append(&mut metadata.diagnostics);
                root_archives.push(RootArchive { relative_path, metadata });
            }
            Err(error) => diagnostics.push(limit_diagnostic(
                "root_archive_unreadable",
                format!(
                    "root archive {} could not be inspected and was skipped: {error}",
                    entry.path().display()
                ),
            )),
        }
    }

    let mut scripts = Vec::new();
    for name in ROOT_SCRIPT_NAMES {
        let relative_path = PathBuf::from(name);
        let Some(content) =
            read_optional_file(path, &relative_path, options, &mut consumed, &mut diagnostics)?
        else {
            continue;
        };
        let Some(kind) = StartupScriptKind::from_path(&relative_path) else {
            continue;
        };
        scripts.push(StartupScript {
            relative_path,
            kind,
            content: String::from_utf8_lossy(&content).into_owned(),
        });
    }

    let mut installations = Vec::new();
    scan_installations(
        path,
        ModLoaderFamily::Forge,
        Path::new("libraries/net/minecraftforge/forge"),
        options,
        &mut consumed,
        &mut diagnostics,
        &mut installations,
    )?;
    scan_installations(
        path,
        ModLoaderFamily::NeoForge,
        Path::new("libraries/net/neoforged/neoforge"),
        options,
        &mut consumed,
        &mut diagnostics,
        &mut installations,
    )?;

    Ok(DirectoryMetadata {
        root_archives,
        installations,
        scripts,
        diagnostics,
    })
}

#[allow(clippy::too_many_arguments)]
fn scan_installations(
    root: &Path,
    family: ModLoaderFamily,
    relative_base: &Path,
    options: &InspectionOptions,
    consumed: &mut u64,
    diagnostics: &mut Vec<InspectionDiagnostic>,
    installations: &mut Vec<ModLoaderInstallation>,
) -> Result<(), ServerInspectionError> {
    let base = root.join(relative_base);
    let entries = match sorted_entries(&base, options) {
        Ok(entries) => entries,
        Err(ServerInspectionError::Open { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    for entry in entries {
        if !entry
            .file_type()
            .map_err(|source| ServerInspectionError::Metadata { path: entry.path(), source })?
            .is_dir()
        {
            continue;
        }
        let coordinate_version = entry.file_name().to_string_lossy().into_owned();
        if coordinate_version.is_empty() {
            continue;
        }
        let relative_directory = relative_base.join(&coordinate_version);
        let windows_args = read_metadata_file(
            root,
            &relative_directory.join("win_args.txt"),
            options,
            consumed,
            diagnostics,
        )?;
        let unix_args = read_metadata_file(
            root,
            &relative_directory.join("unix_args.txt"),
            options,
            consumed,
            diagnostics,
        )?;

        let nested_relative = match family {
            ModLoaderFamily::Forge => PathBuf::from(format!(
                "libraries/net/minecraftforge/fmlloader/{0}/fmlloader-{0}.jar",
                coordinate_version
            )),
            ModLoaderFamily::NeoForge => {
                relative_directory.join(format!("neoforge-{coordinate_version}-universal.jar"))
            }
        };
        let nested_path = root.join(&nested_relative);
        let nested_metadata = if options.max_archive_depth == 0
            || !nested_archive_allowed(&nested_path, options, diagnostics)?
        {
            None
        } else {
            match archive::read_metadata_with_budget(&nested_path, options, consumed) {
                Ok(mut metadata) => {
                    diagnostics.append(&mut metadata.diagnostics);
                    Some(metadata)
                }
                Err(ServerInspectionError::Open { source, .. })
                    if source.kind() == std::io::ErrorKind::NotFound =>
                {
                    None
                }
                Err(error) => return Err(error),
            }
        };

        if windows_args.is_some() || unix_args.is_some() || nested_metadata.is_some() {
            let nested_archive_path = nested_metadata.is_some().then_some(nested_relative);
            installations.push(ModLoaderInstallation {
                family,
                coordinate_version,
                relative_directory,
                windows_args,
                unix_args,
                nested_archive_path,
                nested_metadata,
            });
        }
    }
    Ok(())
}

fn sorted_entries(
    path: &Path,
    options: &InspectionOptions,
) -> Result<Vec<fs::DirEntry>, ServerInspectionError> {
    let mut entries = fs::read_dir(path)
        .map_err(|source| ServerInspectionError::Open { path: path.to_path_buf(), source })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| ServerInspectionError::Open { path: path.to_path_buf(), source })?;
    if entries.len() > options.max_archive_entries {
        return Err(ServerInspectionError::TooManyDirectoryEntries {
            path: path.to_path_buf(),
            count: entries.len(),
            limit: options.max_archive_entries,
        });
    }
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

fn read_metadata_file(
    root: &Path,
    relative_path: &Path,
    options: &InspectionOptions,
    consumed: &mut u64,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Result<Option<MetadataFile>, ServerInspectionError> {
    read_optional_file(root, relative_path, options, consumed, diagnostics).map(|content| {
        content.map(|content| MetadataFile {
            relative_path: relative_path.to_path_buf(),
            content,
        })
    })
}

fn read_optional_file(
    root: &Path,
    relative_path: &Path,
    options: &InspectionOptions,
    consumed: &mut u64,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Result<Option<Vec<u8>>, ServerInspectionError> {
    let path = root.join(relative_path);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => return Ok(None),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ServerInspectionError::Metadata { path, source }),
    };
    if metadata.len() > options.max_metadata_entry_bytes {
        diagnostics.push(limit_diagnostic(
            "metadata_entry_too_large",
            format!(
                "metadata file {} is {} bytes; the inspection limit is {} bytes",
                path.display(),
                metadata.len(),
                options.max_metadata_entry_bytes
            ),
        ));
        return Ok(None);
    }
    let remaining = options.max_total_metadata_bytes.saturating_sub(*consumed);
    if metadata.len() > remaining {
        diagnostics.push(limit_diagnostic(
            "metadata_budget_exceeded",
            format!(
                "reading metadata file {} would exceed the remaining inspection budget of {remaining} bytes",
                path.display()
            ),
        ));
        return Ok(None);
    }

    let mut file = File::open(&path)
        .map_err(|source| ServerInspectionError::Open { path: path.clone(), source })?;
    let mut content = Vec::new();
    file.by_ref()
        .take(
            options
                .max_metadata_entry_bytes
                .min(remaining)
                .saturating_add(1),
        )
        .read_to_end(&mut content)
        .map_err(|source| ServerInspectionError::MetadataRead { path: path.clone(), source })?;
    if content.len() as u64 > options.max_metadata_entry_bytes || content.len() as u64 > remaining {
        diagnostics.push(limit_diagnostic(
            "metadata_budget_exceeded",
            format!("metadata file {} expanded beyond its inspection budget", path.display()),
        ));
        return Ok(None);
    }
    *consumed = consumed.saturating_add(content.len() as u64);
    Ok(Some(content))
}

fn nested_archive_allowed(
    path: &Path,
    options: &InspectionOptions,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Result<bool, ServerInspectionError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => return Ok(false),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(ServerInspectionError::Metadata { path: path.to_path_buf(), source });
        }
    };
    if metadata.len() <= options.max_nested_archive_bytes {
        return Ok(true);
    }
    diagnostics.push(limit_diagnostic(
        "nested_archive_too_large",
        format!(
            "nested archive {} is {} bytes; the inspection limit is {} bytes",
            path.display(),
            metadata.len(),
            options.max_nested_archive_bytes
        ),
    ));
    Ok(false)
}

fn is_root_jar(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".jar")
}

fn root_archive_sort_key(name: &str) -> (u8, String) {
    let lower = name.to_ascii_lowercase();
    if lower == "server.jar" {
        return (0, lower);
    }
    const KNOWN_TOKENS: &[&str] = &[
        "paper",
        "purpur",
        "spigot",
        "craftbukkit",
        "fabric",
        "forge",
        "neoforge",
        "arclight",
        "mohist",
        "magma",
        "velocity",
        "bungeecord",
        "waterfall",
        "sponge",
        "limbo",
    ];
    if KNOWN_TOKENS.iter().any(|token| lower.contains(token)) {
        (1, lower)
    } else {
        (2, lower)
    }
}

fn limit_diagnostic(code: &str, message: String) -> InspectionDiagnostic {
    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: code.to_string(),
        message,
        evidence: Vec::new(),
    }
}
