//! 把一批 region 渲染成 PNG，用于人工比对渲染效果。
//!
//! 用法：`cargo run -p sealantern-feature --example render_regions -- <region 目录> <输出目录>`
//!
//! 只处理目录里实际存在的 `r.<x>.<z>.mca`，不猜坐标。

use std::path::{Path, PathBuf};

use sealantern_feature::world_map::TexturePalette;

fn main() {
    let mut args = std::env::args().skip(1);
    let region_dir = PathBuf::from(args.next().expect("缺少 region 目录"));
    let out_dir = PathBuf::from(args.next().expect("缺少输出目录"));
    // 可选的第三个参数：资源包路径，验证「换材质包，地图跟着变」
    let palette = match args.next() {
        Some(pack) => {
            let palette = TexturePalette::from_resource_pack(Path::new(&pack))
                .unwrap_or_else(|error| panic!("读资源包失败: {error}"));
            println!("用资源包 {pack}，覆盖 {} 张贴图", palette.overridden());
            palette
        }
        None => TexturePalette::builtin(),
    };
    std::fs::create_dir_all(&out_dir).expect("建输出目录");

    let mut rendered = 0usize;
    let mut total_ms = 0u128;

    for entry in std::fs::read_dir(&region_dir).expect("打不开 region 目录") {
        let path = entry.expect("读目录项").path();
        let Some((x, z)) = parse_region_name(&path) else {
            continue;
        };

        let out = out_dir.join(format!("r.{x}.{z}.png"));
        let started = std::time::Instant::now();
        match sealantern_feature::world_map::render_region_png_with(
            &region_dir,
            x,
            z,
            &out,
            &palette,
        ) {
            Ok(info) => {
                let elapsed = started.elapsed().as_millis();
                total_ms += elapsed;
                rendered += 1;
                println!(
                    "r.{x}.{z} -> {} × {}，{} 字节，{} chunk，{} ms",
                    info.pixels, info.pixels, info.bytes, info.rendered_chunks, elapsed
                );
            }
            Err(error) => println!("r.{x}.{z} 失败：{error}"),
        }
    }

    if rendered == 0 {
        println!("没有可渲染的 region");
    } else {
        println!("共 {rendered} 块，总计 {total_ms} ms，平均 {} ms", total_ms / rendered as u128);
    }
}

/// 从 `r.<x>.<z>.mca` 里取出坐标。
fn parse_region_name(path: &Path) -> Option<(isize, isize)> {
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_suffix(".mca")?.strip_prefix("r.")?;
    let (x, z) = stem.split_once('.')?;
    Some((x.parse().ok()?, z.parse().ok()?))
}
