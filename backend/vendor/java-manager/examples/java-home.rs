use java_manager::java_home;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let java_info = java_home().unwrap();
    println!("Name: {}", java_info.name);
    println!("Version: {}", java_info.version);
    println!("Path: {}", java_info.path.display());
    println!("Vendor: {}", java_info.vendor);
    println!("Architecture: {}", java_info.architecture);
    println!("Java Home: {}", java_info.java_home.display());
    Ok(())
}
