use std::fs::File;
use std::io::Read;
use std::path::Path;

use zip::ZipArchive;

use super::error::ServerInspectionError;
use super::model::{DiagnosticSeverity, InspectionDiagnostic};
use super::InspectionOptions;

const MANIFEST_ENTRY: &str = "META-INF/MANIFEST.MF";
const MOJANG_VERSION_ENTRY: &str = "version.json";
pub(super) const VERSIONS_LIST_ENTRY: &str = "META-INF/versions.list";
pub(super) const PATCHES_LIST_ENTRY: &str = "META-INF/patches.list";
pub(super) const LIBRARIES_LIST_ENTRY: &str = "META-INF/libraries.list";

pub(super) struct ArchiveMetadata {
    pub(super) manifest: Option<Vec<u8>>,
    pub(super) mojang_version: Option<Vec<u8>>,
    pub(super) versions_list: Option<Vec<u8>>,
    pub(super) patches_list: Option<Vec<u8>>,
    pub(super) libraries_list: Option<Vec<u8>>,
    pub(super) diagnostics: Vec<InspectionDiagnostic>,
}

pub(super) fn read_metadata(
    path: &Path,
    options: &InspectionOptions,
) -> Result<ArchiveMetadata, ServerInspectionError> {
    let file = File::open(path)
        .map_err(|source| ServerInspectionError::Open { path: path.to_path_buf(), source })?;
    let mut archive = ZipArchive::new(file)
        .map_err(|source| ServerInspectionError::Archive { path: path.to_path_buf(), source })?;
    if archive.len() > options.max_archive_entries {
        return Err(ServerInspectionError::TooManyArchiveEntries {
            path: path.to_path_buf(),
            count: archive.len(),
            limit: options.max_archive_entries,
        });
    }

    let mut diagnostics = Vec::new();
    let mut consumed = 0_u64;
    let manifest = read_optional_entry(
        &mut archive,
        path,
        MANIFEST_ENTRY,
        options,
        &mut consumed,
        &mut diagnostics,
    )?;
    let mojang_version = read_optional_entry(
        &mut archive,
        path,
        MOJANG_VERSION_ENTRY,
        options,
        &mut consumed,
        &mut diagnostics,
    )?;
    let versions_list = read_optional_entry(
        &mut archive,
        path,
        VERSIONS_LIST_ENTRY,
        options,
        &mut consumed,
        &mut diagnostics,
    )?;
    let patches_list = read_optional_entry(
        &mut archive,
        path,
        PATCHES_LIST_ENTRY,
        options,
        &mut consumed,
        &mut diagnostics,
    )?;
    let libraries_list = read_optional_entry(
        &mut archive,
        path,
        LIBRARIES_LIST_ENTRY,
        options,
        &mut consumed,
        &mut diagnostics,
    )?;

    Ok(ArchiveMetadata {
        manifest,
        mojang_version,
        versions_list,
        patches_list,
        libraries_list,
        diagnostics,
    })
}

fn read_optional_entry(
    archive: &mut ZipArchive<File>,
    path: &Path,
    expected_name: &str,
    options: &InspectionOptions,
    consumed: &mut u64,
    diagnostics: &mut Vec<InspectionDiagnostic>,
) -> Result<Option<Vec<u8>>, ServerInspectionError> {
    let Some(index) = find_entry(archive, path, expected_name)? else {
        return Ok(None);
    };
    let mut entry =
        archive
            .by_index(index)
            .map_err(|source| ServerInspectionError::ArchiveEntry {
                path: path.to_path_buf(),
                entry: expected_name.to_string(),
                source,
            })?;
    if entry.size() > options.max_metadata_entry_bytes {
        diagnostics.push(limit_diagnostic(
            "metadata_entry_too_large",
            format!(
                "metadata entry {expected_name} is {} bytes; the inspection limit is {} bytes",
                entry.size(),
                options.max_metadata_entry_bytes
            ),
        ));
        return Ok(None);
    }
    if consumed.saturating_add(entry.size()) > options.max_total_metadata_bytes {
        diagnostics.push(limit_diagnostic(
            "metadata_budget_exceeded",
            format!(
                "reading metadata entry {expected_name} would exceed the total inspection limit of {} bytes",
                options.max_total_metadata_bytes
            ),
        ));
        return Ok(None);
    }

    let remaining_total = options.max_total_metadata_bytes.saturating_sub(*consumed);
    let read_limit = options
        .max_metadata_entry_bytes
        .min(remaining_total)
        .saturating_add(1);
    let mut bytes = Vec::new();
    entry
        .by_ref()
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|source| ServerInspectionError::ArchiveEntryRead {
            path: path.to_path_buf(),
            entry: expected_name.to_string(),
            source,
        })?;
    if bytes.len() as u64 > options.max_metadata_entry_bytes {
        diagnostics.push(limit_diagnostic(
            "metadata_entry_too_large",
            format!(
                "metadata entry {expected_name} expanded beyond the inspection limit of {} bytes",
                options.max_metadata_entry_bytes
            ),
        ));
        return Ok(None);
    }
    if bytes.len() as u64 > remaining_total {
        diagnostics.push(limit_diagnostic(
            "metadata_budget_exceeded",
            format!(
                "metadata entry {expected_name} expanded beyond the remaining total inspection budget of {remaining_total} bytes"
            ),
        ));
        return Ok(None);
    }

    *consumed = consumed.saturating_add(bytes.len() as u64);
    Ok(Some(bytes))
}

fn find_entry(
    archive: &mut ZipArchive<File>,
    path: &Path,
    expected_name: &str,
) -> Result<Option<usize>, ServerInspectionError> {
    for index in 0..archive.len() {
        let entry =
            archive
                .by_index(index)
                .map_err(|source| ServerInspectionError::ArchiveEntry {
                    path: path.to_path_buf(),
                    entry: format!("entry #{index}"),
                    source,
                })?;
        if entry.name().eq_ignore_ascii_case(expected_name) {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn limit_diagnostic(code: &str, message: String) -> InspectionDiagnostic {
    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: code.to_string(),
        message,
        evidence: Vec::new(),
    }
}
