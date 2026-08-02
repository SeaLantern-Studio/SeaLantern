use super::super::model::MavenCoordinate;
use super::maven_coordinate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionListEntry {
    pub(crate) line: usize,
    pub(crate) minecraft_version: String,
    pub(crate) target_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionPatchEntry {
    pub(crate) line: usize,
    pub(crate) minecraft_version: String,
    pub(crate) target_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LibraryListEntry {
    pub(crate) line: usize,
    pub(crate) coordinate: Option<MavenCoordinate>,
    pub(crate) declared_coordinate: String,
    pub(crate) target_path: String,
}

pub(crate) fn parse_versions(content: &[u8]) -> Vec<VersionListEntry> {
    lines(content)
        .into_iter()
        .filter_map(|(line, value)| {
            let fields = split_fields(&value);
            (fields.len() >= 3).then(|| VersionListEntry {
                line,
                minecraft_version: fields[1].to_string(),
                target_path: fields[2].to_string(),
            })
        })
        .filter(|entry| !entry.minecraft_version.is_empty() && !entry.target_path.is_empty())
        .collect()
}

pub(crate) fn parse_version_patches(content: &[u8]) -> Vec<VersionPatchEntry> {
    lines(content)
        .into_iter()
        .filter_map(|(line, value)| {
            let fields = split_fields(&value);
            if fields.len() < 7 || !fields[0].eq_ignore_ascii_case("versions") {
                return None;
            }
            let target_path = fields[6].to_string();
            let minecraft_version = target_path
                .split(['/', '\\'])
                .next()
                .unwrap_or_default()
                .to_string();
            (!minecraft_version.is_empty() && !target_path.is_empty())
                .then_some(VersionPatchEntry { line, minecraft_version, target_path })
        })
        .collect()
}

pub(crate) fn parse_libraries(content: &[u8]) -> Vec<LibraryListEntry> {
    lines(content)
        .into_iter()
        .filter_map(|(line, value)| {
            let fields = split_fields(&value);
            (fields.len() >= 3).then(|| LibraryListEntry {
                line,
                coordinate: maven_coordinate::parse(fields[1]),
                declared_coordinate: fields[1].to_string(),
                target_path: fields[2].to_string(),
            })
        })
        .filter(|entry| !entry.target_path.is_empty())
        .collect()
}

fn lines(content: &[u8]) -> Vec<(usize, String)> {
    String::from_utf8_lossy(content)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            (!line.is_empty()).then_some((index + 1, line.to_string()))
        })
        .collect()
}

fn split_fields(line: &str) -> Vec<&str> {
    let tab_fields = line.split('\t').collect::<Vec<_>>();
    if tab_fields.len() > 1 {
        tab_fields
    } else {
        line.split_whitespace().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_libraries, parse_version_patches, parse_versions};

    #[test]
    fn parses_current_paperclip_lists() {
        let versions = parse_versions(b"hash\t26.2\t26.2/purpur-26.2.jar\ninvalid\n");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].minecraft_version, "26.2");
        assert_eq!(versions[0].target_path, "26.2/purpur-26.2.jar");

        let patches = parse_version_patches(
            b"versions\tinput-hash\tpatch-hash\toutput-hash\t26.2/server-26.2.jar\t26.2/server.patch\t26.2/purpur-26.2.jar\n",
        );
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].minecraft_version, "26.2");

        let libraries = parse_libraries(
            b"hash\torg.purpurmc.purpur:purpur-api:26.2.build.2618-stable\torg/purpurmc/purpur-api.jar\n",
        );
        assert_eq!(libraries.len(), 1);
        assert_eq!(
            libraries[0]
                .coordinate
                .as_ref()
                .map(|coordinate| coordinate.artifact.as_str()),
            Some("purpur-api")
        );
    }

    #[test]
    fn keeps_wildcard_library_entries_for_path_based_fallbacks() {
        let libraries = parse_libraries(b"hash\t*\tspigot-api-26.2-R0.1-SNAPSHOT.jar\n");

        assert_eq!(libraries[0].declared_coordinate, "*");
        assert!(libraries[0].coordinate.is_none());
    }
}
