use super::super::model::MavenCoordinate;

pub(crate) fn parse(value: &str) -> Option<MavenCoordinate> {
    let fields = value.split(':').collect::<Vec<_>>();
    if fields.iter().any(|field| field.trim().is_empty()) {
        return None;
    }

    match fields.as_slice() {
        [group, artifact, version] => Some(MavenCoordinate {
            group: (*group).to_string(),
            artifact: (*artifact).to_string(),
            version: (*version).to_string(),
            classifier: None,
            extension: None,
        }),
        [group, artifact, extension, version] => Some(MavenCoordinate {
            group: (*group).to_string(),
            artifact: (*artifact).to_string(),
            version: (*version).to_string(),
            classifier: None,
            extension: Some((*extension).to_string()),
        }),
        [group, artifact, extension, classifier, version] => Some(MavenCoordinate {
            group: (*group).to_string(),
            artifact: (*artifact).to_string(),
            version: (*version).to_string(),
            classifier: Some((*classifier).to_string()),
            extension: Some((*extension).to_string()),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_standard_and_extended_coordinates() {
        let standard =
            parse("io.papermc.paper:paper-api:26.2.build.87-stable").expect("standard coordinate");
        assert_eq!(standard.group, "io.papermc.paper");
        assert_eq!(standard.artifact, "paper-api");
        assert_eq!(standard.version, "26.2.build.87-stable");

        let extended = parse("example:server:jar:mojmap:1.0").expect("extended coordinate");
        assert_eq!(extended.extension.as_deref(), Some("jar"));
        assert_eq!(extended.classifier.as_deref(), Some("mojmap"));
    }

    #[test]
    fn rejects_wildcards_and_incomplete_coordinates() {
        assert!(parse("*").is_none());
        assert!(parse("paper-api:1.0").is_none());
        assert!(parse("group::1.0").is_none());
    }
}
