use std::collections::BTreeMap;

use super::super::model::ManifestSection;

pub(super) fn attribute<'a>(
    attributes: &'a BTreeMap<String, String>,
    key: &str,
) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn section_by_title<'a>(
    sections: &'a [ManifestSection],
    title: &str,
) -> Option<&'a ManifestSection> {
    sections.iter().find(|section| {
        attribute(&section.attributes, "Implementation-Title")
            .is_some_and(|value| value.eq_ignore_ascii_case(title))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{attribute, section_by_title};
    use crate::provisioning::server_inspection::ManifestSection;

    #[test]
    fn looks_up_non_empty_attributes_case_insensitively() {
        let attributes = BTreeMap::from([
            ("Implementation-Title".to_string(), "Example".to_string()),
            ("Empty".to_string(), "  ".to_string()),
        ]);

        assert_eq!(attribute(&attributes, "implementation-title"), Some("Example"));
        assert_eq!(attribute(&attributes, "empty"), None);
    }

    #[test]
    fn finds_sections_by_implementation_title() {
        let section = ManifestSection {
            name: Some("example/".to_string()),
            attributes: BTreeMap::from([(
                "Implementation-Title".to_string(),
                "Example".to_string(),
            )]),
        };

        assert_eq!(
            section_by_title(&[section], "example").and_then(|value| value.name.as_deref()),
            Some("example/")
        );
    }
}
