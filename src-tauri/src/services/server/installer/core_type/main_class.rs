use super::definitions::CoreType;

pub(in crate::services::server::installer) fn detect_core_type_with_main_class(
    input: &str,
) -> (String, Option<String>) {
    let main_class = read_jar_main_class(input);
    if let Some(ref class_name) = main_class {
        if let Some(core_type) = core_type_from_main_class(class_name) {
            return (core_type.to_string(), Some(class_name.clone()));
        }
    }
    (super::filename::detect_core_type(input), main_class)
}

fn core_type_from_main_class(main_class: &str) -> Option<CoreType> {
    match main_class {
        value if value.starts_with("net.neoforged.serverstarterjar") => Some(CoreType::Neoforge),
        "net.minecraft.server.MinecraftServer" | "net.minecraft.bundler.Main" => {
            Some(CoreType::Vanilla)
        }
        "net.minecraft.client.Main" => Some(CoreType::Unknown),
        "net.minecraftforge.installer.SimpleInstaller" => Some(CoreType::Forge),
        "net.fabricmc.installer.Main" => Some(CoreType::Fabric),
        "net.fabricmc.installer.ServerLauncher" => Some(CoreType::Fabric),
        "io.izzel.arclight.server.Launcher" => Some(CoreType::ArclightForge),
        "catserver.server.CatServerLaunch" | "foxlaunch.FoxServerLauncher" => {
            Some(CoreType::Catserver)
        }
        "org.bukkit.craftbukkit.Main" | "org.bukkit.craftbukkit.bootstrap.Main" => {
            Some(CoreType::Bukkit)
        }
        "io.papermc.paperclip.Main" | "com.destroystokyo.paperclip.Paperclip" => {
            Some(CoreType::Paper)
        }
        "org.leavesmc.leavesclip.Main" => Some(CoreType::Leaves),
        "net.md_5.bungee.Bootstrap" => Some(CoreType::Lightfall),
        "com.mohistmc.MohistMCStart" | "com.mohistmc.MohistMC" => Some(CoreType::Mohist),
        "com.velocitypowered.proxy.Velocity" => Some(CoreType::Velocity),
        _ => None,
    }
}

fn read_jar_main_class(jar_path: &str) -> Option<String> {
    let file = std::fs::File::open(jar_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut manifest = archive.by_name("META-INF/MANIFEST.MF").ok()?;

    let mut bytes = Vec::new();
    use std::io::Read;
    manifest.read_to_end(&mut bytes).ok()?;
    let content = String::from_utf8_lossy(&bytes).to_string();

    find_manifest_main_class(&content)
}

fn find_manifest_main_class(manifest_content: &str) -> Option<String> {
    let mut current_key = String::new();
    let mut current_value = String::new();

    let flush_entry = |key: &str, value: &str| {
        if key.eq_ignore_ascii_case("Main-Class") {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        }
    };

    for line in manifest_content.lines() {
        if line.is_empty() {
            if let Some(value) = flush_entry(&current_key, &current_value) {
                return Some(value);
            }
            current_key.clear();
            current_value.clear();
            continue;
        }

        if line.starts_with(' ') {
            current_value.push_str(line.trim_start());
            continue;
        }

        if let Some(value) = flush_entry(&current_key, &current_value) {
            return Some(value);
        }

        if let Some((key, value)) = line.split_once(':') {
            current_key.clear();
            current_key.push_str(key.trim());
            current_value.clear();
            current_value.push_str(value.trim());
        } else {
            current_key.clear();
            current_value.clear();
        }
    }

    flush_entry(&current_key, &current_value)
}
