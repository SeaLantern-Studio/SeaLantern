//! 从 Minecraft 方块贴图生成静态颜色表。
//!
//! 用法：`cargo run -p sealantern-feature --example gen_block_colors -- <贴图目录> <输出文件>`
//!
//! 对每张 `textures/block/*.png` 取非透明像素的 alpha 加权平均色，按贴图名排序后
//! 写成一份 Rust 常量表。透明像素不参与平均，否则带镂空的方块（栅栏、铁栏杆）
//! 会被周围的透明区域拉淡。
//!
//! 这只是个离线工具：贴图版权归 Mojang，仓库里只放算出来的数值，不放素材。

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// 一张贴图的汇总结果。
struct Average {
    name: String,
    rgb: [u8; 3],
    /// 灰度程度：所有非透明像素 `max - min` 的均值。接近 0 说明是待染色贴图。
    spread: f32,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let texture_dir = args.next().expect("缺少贴图目录");
    let out_path = args.next().expect("缺少输出文件");

    let mut entries: Vec<Average> = fs::read_dir(&texture_dir)
        .expect("打不开贴图目录")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? != "png" {
                return None;
            }
            let name = path.file_stem()?.to_str()?.to_owned();
            average_png(&path, name)
        })
        .collect();

    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut source = String::new();
    source.push_str(
        "//! 方块贴图的平均色，由 `examples/gen_block_colors.rs` 生成，不要手改。\n\
         //!\n\
         //! 键是贴图名（不含扩展名），不是方块名：方块到贴图的映射由调用方决定，\n\
         //! 因为台阶、楼梯这类衍生方块要借用基础材质的贴图。\n\
         //!\n\
         //! 数值来自 1.21.11 的官方贴图，取非透明像素的 alpha 加权平均。\n\
         //! 仓库里只保存算出来的颜色，不保存贴图本身。\n\n",
    );
    source.push_str("/// 一张贴图的平均色。按贴图名字节序排列，供二分查找。\n");
    source.push_str("pub(crate) const TEXTURE_COLORS: &[(&str, [u8; 3])] = &[\n");
    for entry in &entries {
        writeln!(
            source,
            "    (\"{}\", [{}, {}, {}]),",
            entry.name, entry.rgb[0], entry.rgb[1], entry.rgb[2]
        )
        .expect("写字符串");
    }
    source.push_str("];\n");

    if let Some(parent) = Path::new(&out_path).parent() {
        fs::create_dir_all(parent).expect("建输出目录");
    }
    fs::write(&out_path, source).expect("写输出文件");

    let grey = entries.iter().filter(|e| e.spread < 4.0).count();
    println!(
        "{} 张贴图，其中 {} 张接近灰度（大概率要群系染色），写出 {}",
        entries.len(),
        grey,
        out_path
    );
}

/// 读完一张 PNG 并算平均色，读不出来就跳过（损坏或格式不认识的图）。
///
/// 解析逻辑和运行时读资源包用的是同一份（`world_map::average_texture_colour`），
/// 两边算出来的值必须一致，否则内置表和资源包会给出两种颜色。
fn average_png(path: &Path, name: String) -> Option<Average> {
    let bytes = fs::read(path).ok()?;
    let rgb = sealantern_feature::world_map::average_texture_colour(&bytes)?;

    // 灰度程度只用于统计输出，判断有多少张贴图等着群系染色
    let spread = (rgb.iter().copied().max()? - rgb.iter().copied().min()?) as f32;
    Some(Average { name, rgb, spread })
}
