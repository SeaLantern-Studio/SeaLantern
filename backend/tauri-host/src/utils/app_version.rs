pub(crate) fn base_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub(crate) fn display_version() -> String {
    option_env!("SEA_LANTERN_BUILD_VERSION")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| base_version().to_string())
}

#[cfg(test)]
mod tests {
    use super::{base_version, display_version};

    #[test]
    fn display_version_falls_back_to_base_version() {
        assert_eq!(display_version(), base_version());
    }
}
