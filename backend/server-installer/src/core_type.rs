use std::str::FromStr;

use server_core_taxonomy::normalize_core_key as normalize_published_core_key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreType {
    Pumpkin,
    ArclightForge,
    ArclightNeoforge,
    Youer,
    Mohist,
    Catserver,
    Spongeforge,
    ArclightFabric,
    Banner,
    Neoforge,
    Forge,
    Quilt,
    Fabric,
    PufferfishPurpur,
    Pufferfish,
    Spongevanilla,
    Purpur,
    Paper,
    Folia,
    Leaves,
    Leaf,
    Spigot,
    Bukkit,
    VanillaSnapshot,
    Vanilla,
    Nukkitx,
    Bedrock,
    Velocity,
    Bungeecord,
    Lightfall,
    Travertine,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarterInstallMode {
    Standard,
    ForgeLike,
    NeoForgeLike,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerTypeResolution {
    pub api_core_key: String,
    pub docker_type_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarterInstallArgs {
    pub args: Vec<&'static str>,
}

impl CoreType {
    pub const API_CORE_KEYS: &'static [&'static str] = &[
        "pumpkin",
        "paper",
        "purpur",
        "spigot",
        "bukkit",
        "folia",
        "leaves",
        "pufferfish",
        "sponge",
        "arclight-forge",
        "arclight-neoforge",
        "youer",
        "mohist",
        "catserver",
        "neoforge",
        "forge",
        "fabric",
        "quilt",
        "vanilla",
        "velocity",
        "bungeecord",
        "waterfall",
        "lightfall",
        "travertine",
        "flamecord",
        "tuinity",
        "airplane",
        "glowstone",
        "cuberite",
        "minestom",
        "bds",
        "liteloaderbds",
        "levilamina",
        "bdsx",
        "allay",
        "nukkit",
        "powernukkitx",
        "pocketmine",
        "endstone",
    ];

    pub fn all_api_core_keys() -> &'static [&'static str] {
        Self::API_CORE_KEYS
    }

    pub fn to_api_core_key(self) -> Option<&'static str> {
        match self {
            CoreType::Pumpkin => Some("pumpkin"),
            CoreType::ArclightForge => Some("arclight-forge"),
            CoreType::ArclightNeoforge => Some("arclight-neoforge"),
            CoreType::Youer => Some("youer"),
            CoreType::Mohist => Some("mohist"),
            CoreType::Catserver => Some("catserver"),
            CoreType::Spongeforge => Some("forge"),
            CoreType::ArclightFabric => Some("fabric"),
            CoreType::Banner => Some("fabric"),
            CoreType::Neoforge => Some("neoforge"),
            CoreType::Forge => Some("forge"),
            CoreType::Quilt => Some("quilt"),
            CoreType::Fabric => Some("fabric"),
            CoreType::PufferfishPurpur => Some("pufferfish"),
            CoreType::Pufferfish => Some("pufferfish"),
            CoreType::Spongevanilla => Some("sponge"),
            CoreType::Purpur => Some("purpur"),
            CoreType::Paper => Some("paper"),
            CoreType::Folia => Some("folia"),
            CoreType::Leaves => Some("leaves"),
            CoreType::Leaf => Some("leaves"),
            CoreType::Spigot => Some("spigot"),
            CoreType::Bukkit => Some("bukkit"),
            CoreType::VanillaSnapshot => Some("vanilla"),
            CoreType::Vanilla => Some("vanilla"),
            CoreType::Nukkitx => Some("nukkit"),
            CoreType::Bedrock => Some("bds"),
            CoreType::Velocity => Some("velocity"),
            CoreType::Bungeecord => Some("bungeecord"),
            CoreType::Lightfall => Some("lightfall"),
            CoreType::Travertine => Some("travertine"),
            CoreType::Unknown => None,
        }
    }

    pub fn normalize_to_api_core_key(input: &str) -> Option<String> {
        if let Some(published_key) = normalize_published_api_core_key(input) {
            return Some(published_key.to_string());
        }

        Self::from_str(input)
            .ok()
            .and_then(|core_type| core_type.to_api_core_key().map(|value| value.to_string()))
            .or_else(|| {
                let normalized = input.trim().to_ascii_lowercase();
                if normalized.is_empty() {
                    return None;
                }
                Self::all_api_core_keys()
                    .iter()
                    .find(|candidate| **candidate == normalized)
                    .map(|value| (*value).to_string())
            })
    }

    pub fn starter_install_mode(input: &str) -> Option<StarterInstallMode> {
        match Self::normalize_to_api_core_key(input)?.as_str() {
            "neoforge" | "arclight-neoforge" => Some(StarterInstallMode::NeoForgeLike),
            "forge" | "arclight-forge" | "catserver" | "mohist" => {
                Some(StarterInstallMode::ForgeLike)
            }
            _ => Some(StarterInstallMode::Standard),
        }
    }

    pub fn starter_install_args(input: &str) -> Option<StarterInstallArgs> {
        let args = match Self::starter_install_mode(input)? {
            StarterInstallMode::NeoForgeLike => {
                vec!["--install-server", ".", "--server-starter"]
            }
            StarterInstallMode::ForgeLike => vec!["--installServer", "."],
            StarterInstallMode::Standard => vec!["--install-server", "."],
        };

        Some(StarterInstallArgs { args })
    }

    pub fn docker_type_resolution(input: &str) -> Option<DockerTypeResolution> {
        let api_core_key = Self::normalize_to_api_core_key(input)?;
        let docker_type_value = api_core_key.to_ascii_uppercase().replace('-', "_");

        Some(DockerTypeResolution { api_core_key, docker_type_value })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CoreType::Pumpkin => "Pumpkin",
            CoreType::ArclightForge => "Arclight-Forge",
            CoreType::ArclightNeoforge => "Arclight-Neoforge",
            CoreType::Youer => "Youer",
            CoreType::Mohist => "Mohist",
            CoreType::Catserver => "Catserver",
            CoreType::Spongeforge => "Spongeforge",
            CoreType::ArclightFabric => "Arclight-Fabric",
            CoreType::Banner => "Banner",
            CoreType::Neoforge => "Neoforge",
            CoreType::Forge => "Forge",
            CoreType::Quilt => "Quilt",
            CoreType::Fabric => "Fabric",
            CoreType::PufferfishPurpur => "Pufferfish_Purpur",
            CoreType::Pufferfish => "Pufferfish",
            CoreType::Spongevanilla => "Spongevanilla",
            CoreType::Purpur => "Purpur",
            CoreType::Paper => "Paper",
            CoreType::Folia => "Folia",
            CoreType::Leaves => "Leaves",
            CoreType::Leaf => "Leaf",
            CoreType::Spigot => "Spigot",
            CoreType::Bukkit => "Bukkit",
            CoreType::VanillaSnapshot => "Vanilla-Snapshot",
            CoreType::Vanilla => "Vanilla",
            CoreType::Nukkitx => "Nukkitx",
            CoreType::Bedrock => "Bedrock",
            CoreType::Velocity => "Velocity",
            CoreType::Bungeecord => "Bungeecord",
            CoreType::Lightfall => "Lightfall",
            CoreType::Travertine => "Travertine",
            CoreType::Unknown => "Unknown",
        }
    }

    pub fn detect_from_filename(filename: &str) -> Self {
        let filename_lower = filename.to_lowercase();
        for (core_type, keywords) in Self::detection_table() {
            for keyword in *keywords {
                if filename_lower.contains(keyword) {
                    return *core_type;
                }
            }
        }
        CoreType::Unknown
    }

    fn detection_table() -> &'static [(CoreType, &'static [&'static str])] {
        &[
            (CoreType::Pumpkin, &["pumpkin"]),
            (CoreType::ArclightForge, &["arclight-forge"]),
            (CoreType::ArclightNeoforge, &["arclight-neoforge"]),
            (CoreType::Youer, &["youer"]),
            (CoreType::Mohist, &["mohist"]),
            (CoreType::Catserver, &["catserver"]),
            (CoreType::Spongeforge, &["spongeforge"]),
            (CoreType::ArclightFabric, &["arclight-fabric"]),
            (CoreType::Banner, &["banner"]),
            (CoreType::Neoforge, &["neoforge"]),
            (CoreType::Forge, &["forge"]),
            (CoreType::Quilt, &["quilt"]),
            (CoreType::Fabric, &["fabric"]),
            (CoreType::PufferfishPurpur, &["pufferfish_purpur", "pufferfish-purpur"]),
            (CoreType::Pufferfish, &["pufferfish"]),
            (CoreType::Spongevanilla, &["spongevanilla"]),
            (CoreType::Purpur, &["purpur"]),
            (CoreType::Paper, &["paper"]),
            (CoreType::Folia, &["folia"]),
            (CoreType::Leaves, &["leaves"]),
            (CoreType::Leaf, &["leaf"]),
            (CoreType::Spigot, &["spigot"]),
            (CoreType::Bukkit, &["bukkit"]),
            (CoreType::VanillaSnapshot, &["vanilla-snapshot"]),
            (CoreType::Vanilla, &["vanilla"]),
            (CoreType::Nukkitx, &["nukkitx", "nukkit"]),
            (CoreType::Bedrock, &["bedrock"]),
            (CoreType::Velocity, &["velocity"]),
            (CoreType::Bungeecord, &["bungeecord"]),
            (CoreType::Lightfall, &["lightfall"]),
            (CoreType::Travertine, &["travertine"]),
        ]
    }
}

fn normalize_published_api_core_key(input: &str) -> Option<&'static str> {
    match normalize_published_core_key(input)? {
        "arclight_forge" => Some("arclight-forge"),
        "arclight_neoforge" => Some("arclight-neoforge"),
        other => Some(other),
    }
}

impl FromStr for CoreType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pumpkin" => Ok(CoreType::Pumpkin),
            "arclight-forge" => Ok(CoreType::ArclightForge),
            "arclight-neoforge" => Ok(CoreType::ArclightNeoforge),
            "youer" => Ok(CoreType::Youer),
            "mohist" => Ok(CoreType::Mohist),
            "catserver" => Ok(CoreType::Catserver),
            "spongeforge" => Ok(CoreType::Spongeforge),
            "arclight-fabric" => Ok(CoreType::ArclightFabric),
            "banner" => Ok(CoreType::Banner),
            "neoforge" => Ok(CoreType::Neoforge),
            "forge" => Ok(CoreType::Forge),
            "quilt" => Ok(CoreType::Quilt),
            "fabric" => Ok(CoreType::Fabric),
            "pufferfish_purpur" | "pufferfish-purpur" => Ok(CoreType::PufferfishPurpur),
            "pufferfish" => Ok(CoreType::Pufferfish),
            "spongevanilla" => Ok(CoreType::Spongevanilla),
            "purpur" => Ok(CoreType::Purpur),
            "paper" => Ok(CoreType::Paper),
            "folia" => Ok(CoreType::Folia),
            "leaves" => Ok(CoreType::Leaves),
            "leaf" => Ok(CoreType::Leaf),
            "spigot" => Ok(CoreType::Spigot),
            "bukkit" => Ok(CoreType::Bukkit),
            "vanilla-snapshot" => Ok(CoreType::VanillaSnapshot),
            "vanilla" => Ok(CoreType::Vanilla),
            "nukkitx" | "nukkit" => Ok(CoreType::Nukkitx),
            "bedrock" => Ok(CoreType::Bedrock),
            "velocity" => Ok(CoreType::Velocity),
            "bungeecord" => Ok(CoreType::Bungeecord),
            "lightfall" => Ok(CoreType::Lightfall),
            "travertine" => Ok(CoreType::Travertine),
            "unknown" => Ok(CoreType::Unknown),
            _ => Err(format!("Unknown core type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreType, StarterInstallArgs, StarterInstallMode};

    #[test]
    fn normalize_to_api_core_key_accepts_pumpkin_display_name() {
        assert_eq!(CoreType::normalize_to_api_core_key("Pumpkin").as_deref(), Some("pumpkin"));
    }

    #[test]
    fn normalize_to_api_core_key_accepts_published_taxonomy_aliases() {
        assert_eq!(CoreType::normalize_to_api_core_key("Waterfall").as_deref(), Some("waterfall"));
        assert_eq!(
            CoreType::normalize_to_api_core_key("Arclight-Neoforge").as_deref(),
            Some("arclight-neoforge")
        );
        assert_eq!(CoreType::normalize_to_api_core_key("AllayMC").as_deref(), Some("allay"));
        assert_eq!(
            CoreType::normalize_to_api_core_key("bedrock-dedicated-server").as_deref(),
            Some("bds")
        );
    }

    #[test]
    fn normalize_to_api_core_key_canonicalizes_legacy_local_alias_outputs() {
        assert_eq!(CoreType::normalize_to_api_core_key("leaf").as_deref(), Some("leaves"));
        assert_eq!(
            CoreType::normalize_to_api_core_key("pufferfish_purpur").as_deref(),
            Some("pufferfish")
        );
        assert_eq!(CoreType::normalize_to_api_core_key("spongevanilla").as_deref(), Some("sponge"));
        assert_eq!(CoreType::normalize_to_api_core_key("spongeforge").as_deref(), Some("forge"));
        assert_eq!(
            CoreType::normalize_to_api_core_key("arclight-fabric").as_deref(),
            Some("fabric")
        );
        assert_eq!(CoreType::normalize_to_api_core_key("banner").as_deref(), Some("fabric"));
        assert_eq!(
            CoreType::normalize_to_api_core_key("vanilla-snapshot").as_deref(),
            Some("vanilla")
        );
        assert_eq!(CoreType::normalize_to_api_core_key("nukkitx").as_deref(), Some("nukkit"));
        assert_eq!(CoreType::normalize_to_api_core_key("bedrock").as_deref(), Some("bds"));
    }

    #[test]
    fn detect_from_filename_recognizes_pumpkin_executable_name() {
        assert_eq!(CoreType::detect_from_filename("pumpkin-X64-Windows.exe"), CoreType::Pumpkin);
    }

    #[test]
    fn starter_install_mode_classifies_shared_installer_families() {
        assert_eq!(
            CoreType::starter_install_mode("neoforge"),
            Some(StarterInstallMode::NeoForgeLike)
        );
        assert_eq!(
            CoreType::starter_install_mode("Arclight-Neoforge"),
            Some(StarterInstallMode::NeoForgeLike)
        );
        assert_eq!(CoreType::starter_install_mode("forge"), Some(StarterInstallMode::ForgeLike));
        assert_eq!(
            CoreType::starter_install_mode("spongeforge"),
            Some(StarterInstallMode::ForgeLike)
        );
        assert_eq!(CoreType::starter_install_mode("Paper"), Some(StarterInstallMode::Standard));
        assert_eq!(CoreType::starter_install_mode(""), None);
    }

    #[test]
    fn starter_install_args_match_shared_family_cli_contracts() {
        assert_eq!(
            CoreType::starter_install_args("neoforge"),
            Some(StarterInstallArgs {
                args: vec!["--install-server", ".", "--server-starter"],
            })
        );
        assert_eq!(
            CoreType::starter_install_args("catserver"),
            Some(StarterInstallArgs { args: vec!["--installServer", "."] })
        );
        assert_eq!(
            CoreType::starter_install_args("paper"),
            Some(StarterInstallArgs { args: vec!["--install-server", "."] })
        );
        assert_eq!(CoreType::starter_install_args("   "), None);
    }
}

impl std::fmt::Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
