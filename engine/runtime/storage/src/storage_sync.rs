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
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .map_err(|err| object_store::Error::Generic {
                        store: "HttpError",
                        source: Box::new(err),
                    })?;
                let res =
                    client
                        .get(url.clone())
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
