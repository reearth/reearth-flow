use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::Mutex;

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
                    "Failed to strip prefix with err: {e:?}"
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

/// Streams entries into a zip archive as they're produced, instead of writing plain files and re-zipping afterward.
pub struct StreamingZipWriter<T: Write + Seek + Send> {
    writer: Mutex<zip::ZipWriter<T>>,
}

impl<T: Write + Seek + Send> StreamingZipWriter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer: Mutex::new(zip::ZipWriter::new(writer)),
        }
    }

    pub fn write_entry(&self, relative_path: &str, bytes: &[u8]) -> crate::Result<()> {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut writer = self
            .writer
            .lock()
            .map_err(|e| crate::Error::zip(e.to_string()))?;
        writer
            .start_file(relative_path, options)
            .map_err(crate::Error::zip)?;
        writer.write_all(bytes).map_err(crate::Error::zip)?;
        Ok(())
    }

    /// Finalizes the archive's central directory and returns the underlying writer.
    pub fn finish(self) -> crate::Result<T> {
        let writer = self
            .writer
            .into_inner()
            .map_err(|e| crate::Error::zip(e.to_string()))?;
        writer.finish().map_err(crate::Error::zip)
    }
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
        // clean up
        std::fs::remove_file("test.zip").unwrap();
    }

    #[test]
    fn test_streaming_zip_writer() {
        let sink = StreamingZipWriter::new(std::io::Cursor::new(Vec::new()));
        sink.write_entry("a.txt", b"hello").unwrap();
        sink.write_entry("dir/b.txt", b"world").unwrap();
        let cursor = sink.finish().unwrap();

        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("a.txt").unwrap(), &mut contents)
            .unwrap();
        assert_eq!(contents, "hello");

        contents.clear();
        std::io::Read::read_to_string(&mut archive.by_name("dir/b.txt").unwrap(), &mut contents)
            .unwrap();
        assert_eq!(contents, "world");
    }
}
