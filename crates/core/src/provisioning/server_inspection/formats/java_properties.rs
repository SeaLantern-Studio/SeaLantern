use std::collections::BTreeMap;

pub(crate) fn parse(content: &[u8]) -> BTreeMap<String, String> {
    let content = String::from_utf8_lossy(content);
    logical_lines(&content)
        .into_iter()
        .filter_map(|line| split_property(&line))
        .collect()
}

fn logical_lines(content: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for line in content.lines() {
        let line = line.trim_end_matches('\r');
        current.push_str(line);
        let backslashes = current
            .chars()
            .rev()
            .take_while(|character| *character == '\\')
            .count();
        if backslashes % 2 == 1 {
            current.pop();
            continue;
        }
        lines.push(std::mem::take(&mut current));
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn split_property(line: &str) -> Option<(String, String)> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with(['#', '!']) {
        return None;
    }

    let mut escaped = false;
    let mut separator = None;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if matches!(character, '=' | ':') || character.is_whitespace() {
            separator = Some(index);
            break;
        }
    }

    let (key, value) = separator.map_or((line, ""), |separator| {
        let key = &line[..separator];
        let value = line[separator..].trim_start_matches(|character: char| {
            character.is_whitespace() || matches!(character, '=' | ':')
        });
        (key, value)
    });
    let key = unescape(key.trim());
    (!key.is_empty()).then(|| (key, unescape(value.trim())))
}

fn unescape(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match characters.next() {
            Some('t') => output.push('\t'),
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('f') => output.push('\u{000c}'),
            Some(character) => output.push(character),
            None => output.push('\\'),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_installer_and_bootstrap_properties() {
        let properties = parse(
            concat!(
                "# installer\n",
                "fabric-loader-version=0.19.3\n",
                "game-version: 26.2\n",
                "Main-Class net.minecraftforge.bootstrap.ForgeBootstrap\n",
                "Arguments=--launchTarget \\\n",
                "forge_server\n",
            )
            .as_bytes(),
        );

        assert_eq!(properties.get("fabric-loader-version").map(String::as_str), Some("0.19.3"));
        assert_eq!(properties.get("game-version").map(String::as_str), Some("26.2"));
        assert_eq!(
            properties.get("Main-Class").map(String::as_str),
            Some("net.minecraftforge.bootstrap.ForgeBootstrap")
        );
        assert_eq!(
            properties.get("Arguments").map(String::as_str),
            Some("--launchTarget forge_server")
        );
    }
}
