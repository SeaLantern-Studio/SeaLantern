use java_manager::quick_search;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Discovering Java installations...");

    let installations = quick_search()?;

    println!("Found {} Java installation(s):", installations.len());
    println!("{}", "=".repeat(50));

    for (i, java) in installations.iter().enumerate() {
        println!("{}.\tName: \t\t{}", i + 1, java.name);
        println!("\tVersion: \t{}", java.version);
        println!("\tPath: \t\t{}", java.path.display());
        println!("\tVendor: \t{}", java.vendor);
        println!("\tArchitecture: \t{}", java.architecture);
        println!("\tJava Home: \t{}", java.java_home.display());
        println!("{}", "-".repeat(40));
    }

    Ok(())
}
