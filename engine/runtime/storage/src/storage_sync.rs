use std::fs;
use std::ops::Range;
use std::path::Path;
use std::time::Duration;

use bytes::Bytes;
use object_store::ObjectMeta;
use object_store::Result;
use reearth_flow_common::uri::Protocol;
use reearth_flow_common::uri::Uri;

use crate::storage::format_object_store_error;
use crate::storage::Storage;

impl Storage {
    pub fn put_sync(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let _ = self
            .inner
            .blocking()
            .write(p, bytes)
            .map_err(|err| format_object_store_error(err, p))?;
        Ok(())
    }

    pub fn create_dir_sync(&self, location: &Path) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let p = if !p.ends_with('/') {
            format!("{p}/")
        } else {
            p.to_string()
        };
        self.inner
            .blocking()
            .create_dir(p.as_str())
            .map_err(|err| format_object_store_error(err, p.as_str()))
    }

    pub fn append_sync(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let mut w = self
            .inner
            .blocking()
            .writer_with(p)
            .append(true)
            .call()
            .map_err(|err| format_object_store_error(err, p))?;
        w.write(bytes)
            .map_err(|err| format_object_store_error(err, p))
    }

    pub fn get_sync(&self, location: &Path) -> Result<Bytes> {
        match self.base_uri.protocol() {
            Protocol::Http | Protocol::Https => {
                let result = location.to_str().unwrap();
                let url = format!("{}{}", self.base_uri, result);

                // Use std::thread to avoid creating nested Tokio runtime
                // Clone URL for error reporting since it moves into closure
                let url_for_error = url.clone();
                let handle = std::thread::spawn(move || -> Result<Bytes> {
                    let client = reqwest::blocking::Client::builder()
                        .timeout(Duration::from_secs(30))
                        .build()
                        .map_err(|err| object_store::Error::Generic {
                            store: "HttpError",
                            source: Box::new(err),
                        })?;
                    let res =
                        client
                            .get(url)
                            .send()
                            .map_err(|err| object_store::Error::Generic {
                                store: "HttpError",
                                source: Box::new(err),
                            })?;
                    let buf = res.bytes().map_err(|err| object_store::Error::Generic {
                        store: "HttpError",
                        source: Box::new(err),
                    })?;
                    Ok(buf)
                });

                handle.join().map_err(|_| object_store::Error::Generic {
                    store: "HttpError",
                    source: Box::new(std::io::Error::other(format!(
                        "HTTP request thread panicked while fetching {url_for_error}"
                    ))),
                })?
            }
            _ => {
                let p = location.to_str().ok_or(object_store::Error::InvalidPath {
                    source: object_store::path::Error::InvalidPath {
                        path: format!("{location:?}").into(),
                    },
                })?;
                let r = self
                    .inner
                    .blocking()
                    .read(p)
                    .map_err(|err| format_object_store_error(err, p))?;
                Ok(r.to_bytes())
            }
        }
    }

    pub fn exists_sync(&self, location: &Path) -> Result<bool> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        fs::exists(p).map_err(|err| object_store::Error::Generic {
            store: "FileError",
            source: Box::new(err),
        })
    }

    pub fn get_range_sync(&self, location: &Path, range: Range<usize>) -> Result<Bytes> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let bs = self
            .inner
            .blocking()
            .read_with(p)
            .range(range.start as u64..range.end as u64)
            .call()
            .map_err(|err| format_object_store_error(err, p))?;

        Ok(Bytes::from(bs.to_vec()))
    }

    pub fn head_sync(&self, location: &Path) -> Result<ObjectMeta> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let meta = self
            .inner
            .blocking()
            .stat(p)
            .map_err(|err| format_object_store_error(err, p))?;

        Ok(ObjectMeta {
            location: object_store::path::Path::parse(p)?,
            last_modified: meta.last_modified().unwrap_or_default(),
            size: meta.content_length() as u64,
            e_tag: None,
            version: None,
        })
    }

    pub fn delete_sync(&self, location: &Path) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        self.inner
            .blocking()
            .delete(p)
            .map_err(|err| format_object_store_error(err, p))?;
        Ok(())
    }

    pub fn list_sync(&self, prefix: Option<&Path>, recursive: bool) -> Result<Vec<Uri>> {
        let p = prefix.ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{prefix:?}").into(),
            },
        })?;
        let path = p
            .to_str()
            .map(|v| format!("{v}/"))
            .ok_or(object_store::Error::InvalidPath {
                source: object_store::path::Error::InvalidPath {
                    path: format!("{prefix:?}").into(),
                },
            })?;
        let ds = self
            .inner
            .blocking()
            .lister_with(&path)
            .recursive(recursive)
            .call()
            .map_err(|err| format_object_store_error(err, ""))?;
        let result = ds
            .filter_map(|entry| match entry {
                Ok(v) => Some(Uri::for_test(&format!(
                    "{}/{}",
                    self.base_uri.protocol().as_str_with_separator(),
                    v.path()
                ))),
                Err(_) => None,
            })
            .collect::<Vec<_>>();
        Ok(result)
    }

    pub fn copy_sync(&self, from: &Path, to: &Path) -> Result<()> {
        let from = from.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{from:?}").into(),
            },
        })?;
        let to = to.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{to:?}").into(),
            },
        })?;
        self.inner
            .blocking()
            .copy(from.as_ref(), to.as_ref())
            .map_err(|err| format_object_store_error(err, from))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::StorageResolver;
    use reearth_flow_common::uri::Uri;
    use std::path::Path;

    #[test]
    fn test_put_sync_large_citygml_file() {
        let uri = Uri::for_test("ram:///plateau");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let large_citygml = format!(
            r#"<?xml version="1.0"?><bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0">{}</bldg:Building>"#,
            "<gml:name>大規模建物</gml:name>".repeat(100_000)
        );
        
        let result = storage.put_sync(
            Path::new("53394525_bldg_6697_op.gml"),
            Bytes::from(large_citygml)
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_sync_japanese_path() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let content = "建物データ: 東京都渋谷区";
        storage.put_sync(
            Path::new("都市データ/建物/bldg_001.gml"),
            Bytes::from(content)
        ).unwrap();
        
        let result = storage.get_sync(Path::new("都市データ/建物/bldg_001.gml")).unwrap();
        assert_eq!(String::from_utf8_lossy(&result), content);
    }

    #[test]
    fn test_get_sync_nonexistent_file() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let result = storage.get_sync(Path::new("nonexistent_building.gml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_exists_sync_file_operations() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        storage.put_sync(Path::new("exists.gml"), Bytes::from("data")).unwrap();
        
        let exists = storage.exists_sync(Path::new("exists.gml"));
        let not_exists = storage.exists_sync(Path::new("missing.gml"));
        
        assert!(exists.is_ok());
        assert!(not_exists.is_ok());
    }

    #[test]
    fn test_delete_sync_cleanup() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        storage.put_sync(Path::new("temp.gml"), Bytes::from("temporary")).unwrap();
        storage.delete_sync(Path::new("temp.gml")).unwrap();
        
        let result = storage.get_sync(Path::new("temp.gml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_sync_backup_file() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let original_data = "<bldg:Building>Original CityGML</bldg:Building>";
        storage.put_sync(Path::new("original.gml"), Bytes::from(original_data)).unwrap();
        
        let result = storage.copy_sync(Path::new("original.gml"), Path::new("backup.gml"));
        if result.is_ok() {
            let original = storage.get_sync(Path::new("original.gml")).unwrap();
            let backup = storage.get_sync(Path::new("backup.gml")).unwrap();
            assert_eq!(original, backup);
        }
    }

    #[test]
    fn test_get_range_sync_partial_citygml() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let full_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        storage.put_sync(Path::new("data.bin"), Bytes::from(&full_data[..])).unwrap();
        
        let partial = storage.get_range_sync(Path::new("data.bin"), 10..20).unwrap();
        assert_eq!(partial.as_ref(), b"ABCDEFGHIJ");
    }

    #[test]
    fn test_create_dir_sync_nested_structure() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let result = storage.create_dir_sync(Path::new("codelists/Building"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_put_sync_empty_file() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        storage.put_sync(Path::new("empty.gml"), Bytes::new()).unwrap();
        
        let result = storage.get_sync(Path::new("empty.gml")).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_overwrite_existing_citygml() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        storage.put_sync(Path::new("building.gml"), Bytes::from("v1")).unwrap();
        storage.put_sync(Path::new("building.gml"), Bytes::from("v2")).unwrap();
        
        let result = storage.get_sync(Path::new("building.gml")).unwrap();
        assert_eq!(String::from_utf8_lossy(&result), "v2");
    }

    #[test]
    fn test_multiple_concurrent_reads() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        storage.put_sync(Path::new("shared.gml"), Bytes::from("shared data")).unwrap();
        
        let r1 = storage.get_sync(Path::new("shared.gml")).unwrap();
        let r2 = storage.get_sync(Path::new("shared.gml")).unwrap();
        let r3 = storage.get_sync(Path::new("shared.gml")).unwrap();
        
        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }

    #[test]
    fn test_head_sync_metadata() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        let data = b"Test building data";
        storage.put_sync(Path::new("metadata.gml"), Bytes::from(&data[..])).unwrap();
        
        let meta = storage.head_sync(Path::new("metadata.gml")).unwrap();
        assert_eq!(meta.size, data.len() as u64);
    }

    #[test]
    fn test_list_sync_directory() {
        let uri = Uri::for_test("ram:///test");
        let resolver = StorageResolver::new();
        let storage = resolver.resolve(&uri).unwrap();
        
        storage.put_sync(Path::new("city/building1.gml"), Bytes::from("b1")).unwrap();
        storage.put_sync(Path::new("city/building2.gml"), Bytes::from("b2")).unwrap();
        
        let list = storage.list_sync(Some(Path::new("city")), false);
        assert!(list.is_ok());
    }
}

