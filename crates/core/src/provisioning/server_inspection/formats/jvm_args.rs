#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JvmArgumentFile {
    pub(crate) arguments: Vec<String>,
}

impl JvmArgumentFile {
    pub(crate) fn value_after(&self, flag: &str) -> Option<&str> {
        self.arguments
            .windows(2)
            .find(|arguments| arguments[0] == flag)
            .map(|arguments| arguments[1].as_str())
    }

    pub(crate) fn jar_target(&self) -> Option<&str> {
        self.value_after("-jar")
    }
}

pub(crate) fn parse(content: &[u8]) -> JvmArgumentFile {
    let content = String::from_utf8_lossy(content);
    let arguments = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .flat_map(tokenize)
        .collect();
    JvmArgumentFile { arguments }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaped = false;

    for character in input.chars() {
        if escaped {
            token.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                token.push(character);
            }
            continue;
        }
        if quote.is_none() && character.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
            continue;
        }
        token.push(character);
    }
    if escaped {
        token.push('\\');
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_modloader_values_and_forge_shim_targets() {
        let modern = parse(
            b"-classpath\nlibraries/example.jar\nnet.neoforged.fml.startup.Server\n--fml.neoForgeVersion 26.2.0.41-beta\n--fml.mcVersion 26.2\n",
        );
        assert_eq!(modern.value_after("--fml.neoForgeVersion"), Some("26.2.0.41-beta"));
        assert_eq!(modern.value_after("--fml.mcVersion"), Some("26.2"));

        let forge = parse(b"-Dexample=true -jar forge-26.2-65.1.0-shim.jar\n");
        assert_eq!(forge.jar_target(), Some("forge-26.2-65.1.0-shim.jar"));
    }
}
