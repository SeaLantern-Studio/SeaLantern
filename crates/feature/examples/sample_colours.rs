//! 统计一张 PNG 里出现最多的颜色，用于判断配色是否偏离游戏内观感。
//!
//! 用法：`cargo run -p sealantern-feature --example sample_colours -- <png> [前 N 名]`

use std::collections::HashMap;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("缺少 PNG 路径");
    let top: usize = args
        .next()
        .map(|n| n.parse().expect("名次要是整数"))
        .unwrap_or(15);

    let file = std::fs::File::open(&path).expect("打开 PNG");
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().expect("读头");
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).expect("解 PNG");

    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        other => panic!("不支持的颜色类型 {other:?}"),
    };

    let mut histogram: HashMap<[u8; 3], usize> = HashMap::new();
    let mut total = 0usize;
    for pixel in buffer[..info.buffer_size()].chunks_exact(channels) {
        if channels == 4 && pixel[3] == 0 {
            continue;
        }
        *histogram.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
        total += 1;
    }

    let mut ranked: Vec<([u8; 3], usize)> = histogram.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{path}: {total} 个不透明像素，{} 种颜色", ranked.len());
    for (colour, count) in ranked.iter().take(top) {
        let [r, g, b] = *colour;
        let (h, s, v) = hsv(r, g, b);
        println!(
            "  #{r:02X}{g:02X}{b:02X}  ({r:>3},{g:>3},{b:>3})  {:>5.2}%  H{h:>3.0}° S{s:>4.0}% V{v:>3.0}%",
            *count as f64 / total as f64 * 100.0
        );
    }
}

fn hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation * 100.0, max * 100.0)
}
