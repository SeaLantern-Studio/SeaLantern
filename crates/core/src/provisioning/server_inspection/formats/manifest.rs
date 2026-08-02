use std::collections::BTreeMap;

use super::super::model::{ManifestSection, ManifestSummary};

pub(crate) struct ParsedManifest {
    pub(crate) summary: ManifestSummary,
    pub(crate) used_lossy_utf8: bool,
}

impl ParsedManifest {
    pub(crate) fn main_value(&self, key: &str) -> Option<&str> {
        value_ignore_ascii_case(&self.summary.main_attributes, key)
    }
}

pub(crate) fn parse(bytes: &[u8]) -> ParsedManifest {
    let used_lossy_utf8 = std::str::from_utf8(bytes).is_err();
    let content = String::from_utf8_lossy(bytes);
    let mut blocks: Vec<BTreeMap<String, String>> = Vec::new();
    let mut attributes = BTreeMap::new();
    let mut current_key: Option<String> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            flush_attribute(&mut attributes, &mut current_key);
            if !attributes.is_empty() {
                blocks.push(std::mem::take(&mut attributes));
            }
            continue;
        }
        if let Some(continuation) = line.strip_prefix(' ') {
            if let Some(key) = current_key.as_ref() {
                if let Some(value) = attributes.get_mut(key) {
                    value.push_str(continuation);
                }
            }
            continue;
        }

        flush_attribute(&mut attributes, &mut current_key);
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if !key.is_empty() {
                let key = key.to_string();
                attributes.insert(key.clone(), value.trim_start().to_string());
                current_key = Some(key);
            }
        }
    }
    flush_attribute(&mut attributes, &mut current_key);
    if !attributes.is_empty() {
        blocks.push(attributes);
    }

    let mut blocks = blocks.into_iter();
    let main_attributes = blocks.next().unwrap_or_default();
    let sections = blocks
        .map(|mut attributes| {
            let name_key = attributes
                .keys()
                .find(|key| key.eq_ignore_ascii_case("Name"))
                .cloned();
            let name = name_key.and_then(|key| attributes.remove(&key));
            ManifestSection { name, attributes }
        })
        .collect();

    ParsedManifest {
        summary: ManifestSummary { main_attributes, sections },
        used_lossy_utf8,
    }
}

fn flush_attribute(_attributes: &mut BTreeMap<String, String>, current_key: &mut Option<String>) {
    *current_key = None;
}

fn value_ignore_ascii_case<'a>(
    attributes: &'a BTreeMap<String, String>,
    key: &str,
) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn keeps_main_attributes_separate_from_named_sections() {
        let parsed = parse(
            b"Manifest-Version: 1.0\r\nMain-Class: com.example.\r\n Main\r\n\r\nName: forge\r\nImplementation-Version: 65.1.0\r\n\r\n",
        );

        assert_eq!(parsed.main_value("main-class"), Some("com.example.Main"));
        assert_eq!(parsed.main_value("implementation-version"), None);
        assert_eq!(parsed.summary.sections.len(), 1);
        assert_eq!(parsed.summary.sections[0].name.as_deref(), Some("forge"));
        assert_eq!(
            parsed.summary.sections[0]
                .attributes
                .get("Implementation-Version")
                .map(String::as_str),
            Some("65.1.0")
        );
    }
}
