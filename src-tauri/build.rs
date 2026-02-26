use nasm_rs::compile_library_args;
use std::env;
use std::path::Path;

fn main() {
    load_asm();
    tauri_build::build()
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn load_asm() {
    // 获取项目根目录（即 src-tauri 目录）
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let asm_path =
        Path::new(&manifest_dir).join("src/assemblies/panic_report/getregs/x64_linux.asm");

    // 确保文件存在
    if !asm_path.exists() {
        panic!("Assembly file not found: {}", asm_path.display());
    }

    // 编译汇编
    compile_library_args("libget_regs.a", &[asm_path.to_str().unwrap()], &["-felf64"])
        .expect("nasm compilation failed");

    // 获取输出目录并通知链接器
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=get_regs");
}
