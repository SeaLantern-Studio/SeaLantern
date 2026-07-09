use java_manager::{JavaInfo, deep_search, full_search, quick_search};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let installations: Vec<JavaInfo>;

    let mut input = String::new();
    loop {
        print!(
            "Do you want to search for Java installations using quick or deep search? (q: quick search/f: full search/d: deep search) "
        );
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input)?;
        match input.to_lowercase().as_str().trim() {
            "q" => installations = quick_search()?,
            "f" => installations = full_search()?,
            "d" => installations = deep_search()?,
            _ => {
                println!("Invalid input. Please enter 'q', 'f' or 'd'.");
                continue;
            }
        }
        break;
    }

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
