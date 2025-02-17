fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/auth.proto", "proto/document.proto"], &["proto"])?;
    Ok(())
}
