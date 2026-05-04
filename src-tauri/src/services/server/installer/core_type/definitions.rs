use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreType {
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

impl CoreType {
    pub const API_CORE_KEYS: [&'static str; 29] = [
        "paper",
        "purpur",
        "leaf",
        "spigot",
        "bukkit",
        "folia",
        "leaves",
        "pufferfish",
        "pufferfish_purpur",
        "spongevanilla",
        "arclight-forge",
        "arclight-neoforge",
        "youer",
        "mohist",
        "catserver",
        "spongeforge",
        "arclight-fabric",
        "banner",
        "neoforge",
        "forge",
        "fabric",
        "quilt",
        "vanilla",
        "vanilla-snapshot",
        "nukkitx",
        "velocity",
        "bungeecord",
        "lightfall",
        "travertine",
    ];

    pub fn all_api_core_keys() -> &'static [&'static str] {
        &Self::API_CORE_KEYS
    }

    pub fn to_api_core_key(self) -> Option<&'static str> {
        match self {
            CoreType::ArclightForge => Some("arclight-forge"),
            CoreType::ArclightNeoforge => Some("arclight-neoforge"),
            CoreType::Youer => Some("youer"),
            CoreType::Mohist => Some("mohist"),
            CoreType::Catserver => Some("catserver"),
            CoreType::Spongeforge => Some("spongeforge"),
            CoreType::ArclightFabric => Some("arclight-fabric"),
            CoreType::Banner => Some("banner"),
            CoreType::Neoforge => Some("neoforge"),
            CoreType::Forge => Some("forge"),
            CoreType::Quilt => Some("quilt"),
            CoreType::Fabric => Some("fabric"),
            CoreType::PufferfishPurpur => Some("pufferfish_purpur"),
            CoreType::Pufferfish => Some("pufferfish"),
            CoreType::Spongevanilla => Some("spongevanilla"),
            CoreType::Purpur => Some("purpur"),
            CoreType::Paper => Some("paper"),
            CoreType::Folia => Some("folia"),
            CoreType::Leaves => Some("leaves"),
            CoreType::Leaf => Some("leaf"),
            CoreType::Spigot => Some("spigot"),
            CoreType::Bukkit => Some("bukkit"),
            CoreType::VanillaSnapshot => Some("vanilla-snapshot"),
            CoreType::Vanilla => Some("vanilla"),
            CoreType::Nukkitx | CoreType::Bedrock => Some("nukkitx"),
            CoreType::Velocity => Some("velocity"),
            CoreType::Bungeecord => Some("bungeecord"),
            CoreType::Lightfall => Some("lightfall"),
            CoreType::Travertine => Some("travertine"),
            CoreType::Unknown => None,
        }
    }

    pub fn normalize_to_api_core_key(input: &str) -> Option<String> {
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

    pub fn as_str(&self) -> &'static str {
        match self {
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

    pub(crate) fn detection_table() -> &'static [(CoreType, &'static [&'static str])] {
        &[
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

    pub(crate) fn detect_from_filename(filename: &str) -> Self {
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
}

impl FromStr for CoreType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
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

impl std::fmt::Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
