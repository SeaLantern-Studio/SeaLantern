use nasm_rs::compile_library_args;
use std::env;
use std::path::Path;

fn main() {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        load_asm();
    }
    tauri_build::build()
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn load_asm() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let asm_path =
        Path::new(&manifest_dir).join("src/assemblies/panic_report/getregs/x64_linux.asm");

    if !asm_path.exists() {
        panic!("Assembly file not found: {}", asm_path.display());
    }

    compile_library_args("libget_regs.a", &[asm_path.to_str().unwrap()], &["-felf64"])
        .expect("nasm compilation failed");

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=get_regs");
}
