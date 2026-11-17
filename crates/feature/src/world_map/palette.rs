//! 贴图颜色表：内置一份，可选被资源包覆盖。
//!
//! 内置表是从 1.21.11 官方贴图离线算好的平均色（见 [`super::texture_colors`]），
//! 保证用户什么都不配也能出图。资源包覆盖是锦上添花：用户装了什么材质包，
//! 地图就跟着变，不用等我们跟进版本。
//!
//! ## 只换贴图，不解析模型
//!
//! 完整做法要读 `blockstates` → `models` → 贴图路径，才能知道每个方块六个面
//! 各自用的是哪张图（BlueMap 走的就是这条路，代价是一大套 JSON 解析）。这里
//! 故意不做：材质包绝大多数情况就是替换 `textures/block/` 下的同名文件，
//! 按贴图名直接覆盖已经能跟上。真正的模组方块（新增方块类型而非换皮）本来
//! 也不在支持范围内 —— 渲染层只认原版方块名。

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use fastanvil::Rgba;

use super::texture_colors::TEXTURE_COLORS;

/// 资源包里方块贴图所在的相对目录。
const BLOCK_TEXTURES: &str = "assets/minecraft/textures/block/";

/// 一张贴图的平均色。
///
/// 透明像素不参与平均，否则带镂空的贴图（栅栏、铁栏杆、火把）会被周围的
/// 透明区域拉淡，颜色发灰发白。
///
/// 支持 RGB / RGBA / 灰度 / 灰度+alpha / 调色板五种 PNG —— Minecraft 的贴图
/// 里有不少是调色板格式，只认 RGBA 会漏掉一大半。
pub fn average_texture_colour(png_bytes: &[u8]) -> Option<[u8; 3]> {
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().ok()?;

    // 调色板得在 `next_frame` 之前取走：那之后 `reader` 被可变借用了
    let palette = reader.info().palette.clone();
    let trns = reader.info().trns.clone();

    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).ok()?;

    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        png::ColorType::Indexed => 1,
    };
    let indexed = info.color_type == png::ColorType::Indexed;

    let mut sum = [0u64; 3];
    let mut weight = 0u64;

    for pixel in buffer[..info.buffer_size()].chunks_exact(channels) {
        // 调色板图里每个字节是索引而不是灰度值
        let (rgb, alpha) = if indexed {
            let index = pixel[0] as usize;
            let Some(table) = palette.as_ref() else {
                continue;
            };
            let offset = index * 3;
            if offset + 2 >= table.len() {
                continue;
            }
            let alpha = trns
                .as_ref()
                .and_then(|entries| entries.get(index).copied())
                .unwrap_or(255);
            ([table[offset], table[offset + 1], table[offset + 2]], alpha as u64)
        } else {
            match channels {
                4 => ([pixel[0], pixel[1], pixel[2]], pixel[3] as u64),
                3 => ([pixel[0], pixel[1], pixel[2]], 255),
                2 => ([pixel[0], pixel[0], pixel[0]], pixel[1] as u64),
                _ => ([pixel[0], pixel[0], pixel[0]], 255),
            }
        };
        if alpha == 0 {
            continue;
        }
        for (slot, channel) in sum.iter_mut().zip(rgb) {
            *slot += channel as u64 * alpha;
        }
        weight += alpha;
    }

    if weight == 0 {
        return None;
    }
    Some([(sum[0] / weight) as u8, (sum[1] / weight) as u8, (sum[2] / weight) as u8])
}

/// 渲染时用的贴图颜色表。
///
/// 查不到覆盖色就退回内置表。两者都查不到说明这方块不在原版贴图里
/// （模组方块），由调用方决定兜底颜色。
#[derive(Debug, Clone, Default)]
pub struct TexturePalette {
    /// 资源包覆盖的贴图色，键是贴图名（不含扩展名）。
    overrides: HashMap<String, [u8; 3]>,
}

impl TexturePalette {
    /// 只用内置表。
    pub fn builtin() -> Self {
        Self::default()
    }

    /// 从资源包目录或 zip 读取覆盖色。
    ///
    /// 打不开、不是 zip、里面没有 `assets/minecraft/textures/block/` 都算错误，
    /// 由调用方决定要不要降级到 [`Self::builtin`]。单张贴图解不出来只是跳过，
    /// 不让一张坏图毁掉整个包。
    pub fn from_resource_pack(root: &Path) -> Result<Self, ResourcePackError> {
        let mut overrides = HashMap::new();

        let metadata = fs::metadata(root).map_err(|source| ResourcePackError::Open {
            path: root.display().to_string(),
            source,
        })?;

        if metadata.is_dir() {
            load_from_dir(root, &mut overrides).map_err(|source| ResourcePackError::Read {
                path: root.display().to_string(),
                source,
            })?;
        } else {
            load_from_zip(root, &mut overrides).map_err(|reason| ResourcePackError::Archive {
                path: root.display().to_string(),
                reason,
            })?;
        }

        Ok(Self { overrides })
    }

    /// 覆盖了多少张贴图。为 0 说明这个包对这个版本没意义。
    pub fn overridden(&self) -> usize {
        self.overrides.len()
    }

    /// 这张表有没有被资源包覆盖过。没有的话就是纯内置表。
    pub fn is_builtin(&self) -> bool {
        self.overrides.is_empty()
    }

    /// 查一张贴图的平均色。先看覆盖，再看内置。
    pub fn lookup(&self, texture: &str) -> Option<Rgba> {
        if let Some([r, g, b]) = self.overrides.get(texture) {
            return Some([*r, *g, *b, 255]);
        }
        TEXTURE_COLORS
            .binary_search_by(|(name, _)| (*name).cmp(texture))
            .ok()
            .map(|index| {
                let [r, g, b] = TEXTURE_COLORS[index].1;
                [r, g, b, 255]
            })
    }
}

/// 资源包读取失败的原因。
#[derive(Debug)]
pub enum ResourcePackError {
    /// 路径不存在或没权限。
    Open {
        /// 资源包路径。
        path: String,
        /// 底层 IO 错误。
        source: std::io::Error,
    },
    /// 目录形式的资源包读不动。
    Read {
        /// 资源包路径。
        path: String,
        /// 底层 IO 错误。
        source: std::io::Error,
    },
    /// zip 形式的资源包打不开。
    Archive {
        /// 资源包路径。
        path: String,
        /// 失败原因。
        reason: String,
    },
}

impl std::fmt::Display for ResourcePackError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open { path, source } => write!(formatter, "打不开资源包 {path}: {source}"),
            Self::Read { path, source } => write!(formatter, "读资源包 {path} 失败: {source}"),
            Self::Archive { path, reason } => write!(formatter, "资源包 {path} 解析失败: {reason}"),
        }
    }
}

impl std::error::Error for ResourcePackError {}

/// 目录形式的资源包（开发中的包通常还没打包）。
fn load_from_dir(root: &Path, overrides: &mut HashMap<String, [u8; 3]>) -> std::io::Result<()> {
    let texture_dir = root.join(BLOCK_TEXTURES);
    // 没有这个目录不算错：可能是只改了模型或音效的包，对地图没影响
    let Ok(entries) = fs::read_dir(&texture_dir) else {
        return Ok(());
    };

    for entry in entries {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "png") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        if let Some(colour) = average_texture_colour(&bytes) {
            overrides.insert(stem.to_owned(), colour);
        }
    }
    Ok(())
}

/// zip 形式的资源包。
fn load_from_zip(root: &Path, overrides: &mut HashMap<String, [u8; 3]>) -> Result<(), String> {
    let file = fs::File::open(root).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = entry.name().to_owned();
        let Some(rest) = name.strip_prefix(BLOCK_TEXTURES) else {
            continue;
        };
        // 只要这一层的贴图，子目录（比如 `block/entity/`）跳过
        let Some(stem) = rest.strip_suffix(".png") else {
            continue;
        };
        if stem.contains('/') {
            continue;
        }

        let mut bytes = Vec::new();
        if entry.read_to_end(&mut bytes).is_err() {
            continue;
        }
        if let Some(colour) = average_texture_colour(&bytes) {
            overrides.insert(stem.to_owned(), colour);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    /// 造一张纯色 PNG。
    fn solid_png(rgb: [u8; 3], alpha: u8) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buffer, 16, 16);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("写 PNG 头");
            let pixels: Vec<u8> = std::iter::repeat([rgb[0], rgb[1], rgb[2], alpha])
                .take(16 * 16)
                .flatten()
                .collect();
            writer.write_image_data(&pixels).expect("写图像数据");
        }
        buffer
    }

    fn write_zip(path: &Path, entries: &[(&str, Vec<u8>)]) {
        let file = std::fs::File::create(path).expect("建 zip");
        let mut archive = zip::ZipWriter::new(file);
        for (name, content) in entries {
            archive
                .start_file(*name, zip::write::FileOptions::<()>::default())
                .expect("建条目");
            archive.write_all(content).expect("写条目");
        }
        archive.finish().expect("收尾");
    }

    /// zip 形式的资源包能覆盖同名贴图，没覆盖的继续走内置表。
    #[test]
    fn zip_pack_overrides_matching_textures() {
        let dir = tempfile::tempdir().expect("临时目录");
        let path = dir.path().join("pack.zip");
        write_zip(
            &path,
            &[
                ("assets/minecraft/textures/block/sand.png", solid_png([255, 0, 0], 255)),
                // 子目录里的不认，只收这一层
                ("assets/minecraft/textures/block/entity/chest.png", solid_png([0, 255, 0], 255)),
                // 非 png 直接跳过
                ("pack.mcmeta", b"{}".to_vec()),
            ],
        );

        let palette = TexturePalette::from_resource_pack(&path).expect("读资源包");
        assert_eq!(palette.overridden(), 1, "只该收到底层那一张");
        assert_eq!(palette.lookup("sand"), Some([255, 0, 0, 255]));
        assert_eq!(
            palette.lookup("stone"),
            TexturePalette::builtin().lookup("stone"),
            "没被覆盖的贴图要用内置值"
        );
    }

    /// 目录形式的资源包（开发中的包还没打包）走同一条路。
    #[test]
    fn directory_pack_overrides_matching_textures() {
        let dir = tempfile::tempdir().expect("临时目录");
        let textures = dir.path().join("assets/minecraft/textures/block");
        fs::create_dir_all(&textures).expect("建目录");
        fs::write(textures.join("stone.png"), solid_png([10, 20, 30], 255)).expect("写贴图");

        let palette = TexturePalette::from_resource_pack(dir.path()).expect("读资源包");
        assert_eq!(palette.overridden(), 1);
        assert_eq!(palette.lookup("stone"), Some([10, 20, 30, 255]));
    }

    /// 全透明的贴图不算数，不能把方块涂成黑。
    #[test]
    fn fully_transparent_texture_is_skipped() {
        let dir = tempfile::tempdir().expect("临时目录");
        let textures = dir.path().join("assets/minecraft/textures/block");
        fs::create_dir_all(&textures).expect("建目录");
        fs::write(textures.join("glass.png"), solid_png([0, 0, 0], 0)).expect("写贴图");

        let palette = TexturePalette::from_resource_pack(dir.path()).expect("读资源包");
        assert_eq!(palette.overridden(), 0, "全透明贴图应该被跳过");
        assert!(palette.lookup("glass").is_some(), "跳过之后仍能查内置值");
    }

    /// 路径不存在要报错，不能 panic。
    #[test]
    fn missing_pack_is_an_error() {
        let dir = tempfile::tempdir().expect("临时目录");
        let error = TexturePalette::from_resource_pack(&dir.path().join("nope.zip"))
            .expect_err("不存在的路径应该报错");
        assert!(matches!(error, ResourcePackError::Open { .. }), "{error}");
    }

    /// 不是 zip 的文件要报错，不能 panic。
    #[test]
    fn broken_archive_is_an_error() {
        let dir = tempfile::tempdir().expect("临时目录");
        let path = dir.path().join("bad.zip");
        fs::write(&path, b"not a zip at all").expect("写坏文件");

        let error = TexturePalette::from_resource_pack(&path).expect_err("坏 zip 应该报错");
        assert!(matches!(error, ResourcePackError::Archive { .. }), "{error}");
    }
}
