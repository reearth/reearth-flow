use std::io::{Read, Write};
use std::path::Path;

use walkdir::WalkDir;

pub fn write<T, P>(writer: T, directory: P) -> crate::Result<()>
where
    T: Write + std::io::Seek,
    P: AsRef<Path> + Clone,
{
    let mut zip_writer = zip::ZipWriter::new(writer);
    let walkdir = WalkDir::new(directory.clone());

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut buffer = Vec::new();
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        let relative_path = match path.strip_prefix(&directory) {
            Ok(p) => p,
            Err(e) => {
                return Err(crate::Error::zip(format!(
                    "Failed to strip prefix with err: {:?}",
                    e
                )))
            }
        };
        let path_as_string = relative_path.to_string_lossy().replace('\\', "/");

        if path.is_file() {
            zip_writer
                .start_file(path_as_string, options)
                .map_err(crate::Error::zip)?;
            let mut f = std::fs::File::open(path).map_err(crate::Error::zip)?;
            f.read_to_end(&mut buffer).map_err(crate::Error::zip)?;
            zip_writer.write_all(&buffer).map_err(crate::Error::zip)?;
            buffer.clear();
        } else if path.is_dir() && !path_as_string.is_empty() {
            zip_writer
                .add_directory(path_as_string, options)
                .map_err(crate::Error::zip)?;
        }
    }
    zip_writer.finish().map_err(crate::Error::zip)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::Builder;

    #[test]
    fn test_write() {
        let temp_dir = Builder::new().prefix("foobar").tempdir_in(".").unwrap();
        let file_path = temp_dir.path().join("file");
        std::fs::File::create(file_path).unwrap();
        // Create a subdirectory
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        // Create a file in the subdirectory
        let subfile_path = subdir.join("subfile");
        std::fs::File::create(subfile_path).unwrap();
        assert!(write(std::fs::File::create("test.zip").unwrap(), temp_dir.path(),).is_ok());
    }
}
