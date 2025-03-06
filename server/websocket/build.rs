use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("src/thrift")?;

    let output = Command::new("thrift")
        .args(["--gen", "rs", "-out", "src/thrift", "thrift/auth.thrift"])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=Failed to generate Thrift code: {}", error);
        return Err(error.to_string().into());
    }

    println!("cargo:rerun-if-changed=thrift/auth.thrift");

    Ok(())
}
