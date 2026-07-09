<div align="center">

# java-manager

![Rust.Crate](https://img.shields.io/badge/Crate-java--manager-brightgreen?style=for-the-badge&logo=Rust&logoColor=orange
)
[![Crates.io](https://img.shields.io/crates/v/java-manager.svg?style=for-the-badge)](https://crates.io/crates/java-manager)
[![License](https://img.shields.io/github/license/TaimWay/java-manager?style=for-the-badge&logo=apachelucene&logoColor=white
)](https://github.com/TaimWay/java-manager/blob/main/LICENSE-APACHE.txt)

[![Github](https://img.shields.io/badge/Github-TaimWay%2Fjava--manager-black?style=for-the-badge&logo=Github&logoColor=white
)](https://github.com/TaimWay/java-manager)
[![Author](https://img.shields.io/badge/Author-TaimWay-green?style=for-the-badge&logo=devdotto&logoColor=white
)](https://github.com/TaimWay)
![DevState](https://img.shields.io/badge/DevState-Debug%2FIndev-red?style=for-the-badge&logo=devbox&logoColor=red
)

A comprehensive Rust library and command-line tool for discovering, managing, and interacting with Java installations.

</div>

---

> **The project is currently under development. All bugs related to the project can be reported by submitting issues on GitHub, and we will regularly fix the reported problems**

## Features

- **Cross‑platform** – Works on Windows, macOS, and Linux/Unix.
- **Java discovery** – Find Java installations via `PATH`, `JAVA_HOME`, deep system scans (Everything SDK on Windows, walkdir on Linux), or full system scan (registry + keyword BFS + Microsoft Store + `where` command).
- **Detailed metadata** – Extract version, vendor, architecture, and the location of the `java` executable and `JAVA_HOME`.
- **Execution control** – Run Java programs (JAR or main class) with configurable memory limits, arguments, and I/O redirection.
- **Error handling** – Comprehensive error types for path issues, I/O, command execution, and process failures.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
java-manager = "0.2"
```

Or use the `cargo` command:

```bash
cargo add java-manager
```

## Usage

### Locate Java installations

```rust
use java_manager::{quick_search, deep_search, full_search, java_home};

// Quick search: look for 'java' in every directory in PATH
let javas = quick_search()?;
for java in javas {
    println!("Found Java at {} (version {})", java.path.display(), java.version);
}

// Deep search: Everything SDK (Windows) or walkdir (Linux)
let all_javas = deep_search()?;

// Full search: multiple scanners without external dependencies
let all_javas = full_search()?;

// Check JAVA_HOME environment variable
if let Some(java) = java_home() {
    println!("JAVA_HOME points to Java version {}", java.version);
}
```

### Execute a Java program

```rust
use java_manager::{JavaRunner, JavaRedirect};

let java = java_home().expect("JAVA_HOME not set");

// Run a JAR file
JavaRunner::new()
    .java(java.clone())
    .jar("myapp.jar")
    .min_memory(256 * 1024 * 1024)   // 256 MB
    .max_memory(1024 * 1024 * 1024)  // 1 GB
    .arg("--server")
    .redirect(JavaRedirect::new().output("out.log").error("err.log"))
    .execute()?;

// Or run a main class
JavaRunner::new()
    .java(java)
    .main_class("com.example.Main")
    .arg("arg1")
    .arg("arg2")
    .execute()?;
```

### Get metadata from a specific Java path

```rust
use java_manager::JavaInfo;

let info = JavaInfo::new("/usr/lib/jvm/java-11-openjdk/bin/java".into())?;
println!("Name: {}", info.name);
println!("Version: {}", info.version);
println!("Vendor: {}", info.vendor);
println!("Architecture: {}", info.architecture);
println!("JAVA_HOME: {}", info.java_home.display());
```

## Platform Notes

Three search functions serve different needs:
- **`quick_search()`** — Walks `PATH` on all platforms. Fastest, catches the default Java.
- **`deep_search()`** — Windows: [Everything SDK](https://www.voidtools.com/) (Everything must be installed + running). Linux: same as `full_search()`.
- **`full_search()`** — No external dependencies. Windows: registry + keyword BFS + Microsoft Store + `where` command. Linux: walks common directories + `~/.minecraft/runtime` with keyword filtering.
- **macOS**: Currently supports the same methods as Linux (will be enhanced).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.