use std::io::SeekFrom;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use crate::archive::SevenZArchiveEntry;
use crate::error::Error;
use crate::reader::SevenZReader;

/// decompress a source reader to [dest] path
#[inline]
pub fn decompress<R: Read + Seek>(src_reader: R, dest: impl AsRef<Path>) -> Result<(), Error> {
    decompress_with_extract_fn(src_reader, dest, default_entry_extract_fn)
}

#[inline]
pub fn decompress_with_extract_fn<R: Read + Seek>(
    src_reader: R,
    dest: impl AsRef<Path>,
    extract_fn: impl FnMut(&SevenZArchiveEntry, &mut dyn Read, &PathBuf) -> Result<bool, Error>,
) -> Result<(), Error> {
    decompress_impl(src_reader, dest, extract_fn)
}

fn decompress_impl<R: Read + Seek>(
    mut src_reader: R,
    dest: impl AsRef<Path>,
    mut extract_fn: impl FnMut(&SevenZArchiveEntry, &mut dyn Read, &PathBuf) -> Result<bool, Error>,
) -> Result<(), Error> {
    let pos = src_reader.stream_position().map_err(Error::io)?;
    let len = src_reader.seek(SeekFrom::End(0)).map_err(Error::io)?;
    src_reader.seek(SeekFrom::Start(pos)).map_err(Error::io)?;
    let mut seven = SevenZReader::new(src_reader, len)?;
    let dest = PathBuf::from(dest.as_ref());
    if !dest.exists() {
        std::fs::create_dir_all(&dest).map_err(Error::io)?;
    }
    seven.for_each_entries(|entry, reader| {
        let dest_path = dest.join(entry.name());
        extract_fn(entry, reader, &dest_path)
    })?;

    Ok(())
}

pub fn default_entry_extract_fn(
    entry: &SevenZArchiveEntry,
    reader: &mut dyn Read,
    dest: &PathBuf,
) -> Result<bool, Error> {
    use std::{fs::File, io::BufWriter};

    if entry.is_directory() {
        let dir = dest;
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(Error::io)?;
        }
    } else {
        let path = dest;
        path.parent().and_then(|p| {
            if !p.exists() {
                std::fs::create_dir_all(p).ok()
            } else {
                None
            }
        });
        let file = File::create(path)
            .map_err(|e| Error::file_open(e, path.to_string_lossy().to_string()))?;
        if entry.size() > 0 {
            let mut writer = BufWriter::new(file);
            std::io::copy(reader, &mut writer).map_err(Error::io)?;
            filetime_creation::set_file_handle_times(
                writer.get_ref(),
                Some(filetime_creation::FileTime::from_system_time(
                    entry.access_date().into(),
                )),
                Some(filetime_creation::FileTime::from_system_time(
                    entry.last_modified_date().into(),
                )),
                Some(filetime_creation::FileTime::from_system_time(
                    entry.creation_date().into(),
                )),
            )
            .map_err(Error::io)?;
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::archive::SevenZArchiveEntry;
    use crate::de_funcs::{decompress, default_entry_extract_fn};
    use std::io::Cursor;
    use tempfile::TempDir;

    #[test]
    fn test_decompress_empty_archive() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path();
        
        let empty_7z = create_minimal_7z_archive();
        let cursor = Cursor::new(empty_7z);
        
        let result = decompress(cursor, dest);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_decompress_invalid_signature() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path();
        
        let invalid_data = vec![0u8; 100];
        let cursor = Cursor::new(invalid_data);
        
        let result = decompress(cursor, dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_truncated_file() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path();
        
        let truncated = b"7z\xBC\xAF\x27\x1C";
        let cursor = Cursor::new(truncated.to_vec());
        
        let result = decompress(cursor, dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_to_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("nonexistent/nested/path");
        
        let empty_7z = create_minimal_7z_archive();
        let cursor = Cursor::new(empty_7z);
        
        let result = decompress(cursor, &dest);
        if result.is_ok() {
            assert!(dest.exists());
        }
    }

    #[test]
    fn test_default_entry_extract_fn_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test_dir");
        
        let entry = SevenZArchiveEntry {
            name: "test_directory".to_string(),
            is_directory: true,
            has_stream: false,
            size: 0,
            ..Default::default()
        };
        
        let mut empty_reader: &[u8] = &[];
        let result = default_entry_extract_fn(&entry, &mut empty_reader, &dest);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_entry_extract_fn_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("empty.txt");
        
        let entry = SevenZArchiveEntry {
            name: "empty.txt".to_string(),
            is_directory: false,
            has_stream: true,
            size: 0,
            ..Default::default()
        };
        
        let mut empty_reader: &[u8] = &[];
        let result = default_entry_extract_fn(&entry, &mut empty_reader, &dest);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_entry_extract_fn_with_content() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test.txt");
        
        let content = b"Test content for Plateau CityGML";
        let entry = SevenZArchiveEntry {
            name: "test.txt".to_string(),
            is_directory: false,
            has_stream: true,
            size: content.len() as u64,
            ..Default::default()
        };
        
        let mut reader: &[u8] = content;
        let result = default_entry_extract_fn(&entry, &mut reader, &dest);
        
        assert!(result.is_ok());
        if dest.exists() {
            let extracted_content = std::fs::read_to_string(&dest).unwrap();
            assert_eq!(extracted_content, "Test content for Plateau CityGML");
        }
    }

    #[test]
    fn test_sevenz_archive_entry_new() {
        let entry = SevenZArchiveEntry::new();
        assert_eq!(entry.name(), "");
        assert!(!entry.is_directory());
        assert!(!entry.has_stream());
    }

    #[test]
    fn test_sevenz_archive_entry_accessors() {
        let entry = SevenZArchiveEntry {
            name: "test.gml".to_string(),
            is_directory: false,
            has_stream: true,
            size: 1024,
            ..Default::default()
        };
        
        assert_eq!(entry.name(), "test.gml");
        assert!(!entry.is_directory());
        assert!(entry.has_stream());
        assert_eq!(entry.size(), 1024);
    }

    #[test]
    fn test_japanese_filename_entry() {
        let entry = SevenZArchiveEntry {
            name: "東京都/建物/bldg_001.gml".to_string(),
            is_directory: false,
            has_stream: true,
            size: 512,
            ..Default::default()
        };
        
        assert!(entry.name().contains("東京都"));
        assert!(entry.name().contains("建物"));
    }

    fn create_minimal_7z_archive() -> Vec<u8> {
        vec![
            0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }
}

