use java_manager::{JavaRedirect, JavaRunner, java_home};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let java = java_home().unwrap();

    let _ = java.execute("-version");
    let _ = java.execute_with_output("-jar examples/app.jar");
    let _ = java.execute_with_error("-jar examples/error.jar");

    let redirect = JavaRedirect::new()
        .output("out.log")
        .error("err.log")
        .input("data.txt");

    JavaRunner::new()
        .java(java)
        .jar("examples/myapp.jar")
        .min_memory(512 * 1024 * 1024) // 512 MB
        .max_memory(2 * 1024 * 1024 * 1024) // 2 GB
        .arg("--verbose")
        .redirect(redirect)
        .execute()?;

    Ok(())
}
