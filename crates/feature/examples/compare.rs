//! 把多张 PNG 按同一区域裁剪后横向拼在一起，用于人工比对渲染效果。
//!
//! 用法：`cargo run -p sealantern-feature --example compare -- <输出> <放大倍数> <输入...>`
//!
//! 裁剪区取所有输入非透明像素的并集，这样几版之间位置严格对齐，不会有谁被
//! 单独裁掉一块导致看起来「地形变了」。放大用最近邻，地图是像素画，别插值糊掉。

use std::path::PathBuf;

struct Image {
    width: usize,
    height: usize,
    pixels: Vec<[u8; 4]>,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out_path = PathBuf::from(args.next().expect("缺少输出路径"));
    let scale: usize = args
        .next()
        .expect("缺少放大倍数")
        .parse()
        .expect("倍数要是整数");
    let inputs: Vec<PathBuf> = args.map(PathBuf::from).collect();
    assert!(inputs.len() >= 2, "至少要两张图");

    let images: Vec<Image> = inputs.iter().map(|path| load(path)).collect();

    // 非透明像素的并集
    let mut left = usize::MAX;
    let mut top = usize::MAX;
    let mut right = 0usize;
    let mut bottom = 0usize;
    for image in &images {
        let (l, t, r, b) = bounds(image);
        left = left.min(l);
        top = top.min(t);
        right = right.max(r);
        bottom = bottom.max(b);
    }
    assert!(left < right && top < bottom, "所有图都是空的");
    let crop_width = right - left;
    let crop_height = bottom - top;

    const GAP: usize = 4;
    let out_width = images.len() * crop_width * scale + (images.len() - 1) * GAP;
    let out_height = crop_height * scale;

    // 间隙填深灰，和地图内容的浅色拉开
    let mut out = vec![[40u8, 40, 44, 255]; out_width * out_height];

    for (index, image) in images.iter().enumerate() {
        let base_x = index * (crop_width * scale + GAP);
        for y in 0..crop_height {
            for x in 0..crop_width {
                let source = image.pixels[(top + y) * image.width + left + x];
                for dy in 0..scale {
                    for dx in 0..scale {
                        let target = (y * scale + dy) * out_width + base_x + x * scale + dx;
                        out[target] = source;
                    }
                }
            }
        }
    }

    write_png(&out_path, out_width, out_height, &out);
    println!(
        "裁剪 {crop_width}×{crop_height}（从 {left},{top}），{} 张并排，放大 {scale}×，输出 {out_width}×{out_height}",
        images.len()
    );
}

fn load(path: &PathBuf) -> Image {
    let file = std::fs::File::open(path).expect("打开输入图");
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().expect("读 PNG 头");
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).expect("解 PNG");
    assert_eq!(info.color_type, png::ColorType::Rgba, "输入图应是 RGBA");
    let pixels = buffer[..info.buffer_size()]
        .chunks_exact(4)
        .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
        .collect();
    Image {
        width: info.width as usize,
        height: info.height as usize,
        pixels,
    }
}

/// 非透明像素的包围盒（左、上、右开区间、下开区间）。
fn bounds(image: &Image) -> (usize, usize, usize, usize) {
    let mut left = image.width;
    let mut top = image.height;
    let mut right = 0;
    let mut bottom = 0;
    for y in 0..image.height {
        for x in 0..image.width {
            if image.pixels[y * image.width + x][3] != 0 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
    }
    (left, top, right, bottom)
}

fn write_png(path: &PathBuf, width: usize, height: usize, pixels: &[[u8; 4]]) {
    let file = std::fs::File::create(path).expect("建输出图");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("写 PNG 头");
    let flat: Vec<u8> = pixels.iter().flatten().copied().collect();
    writer.write_image_data(&flat).expect("写 PNG 数据");
}
