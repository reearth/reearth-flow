use std::fs;
use std::ops::Range;
use std::path::Path;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use bytes::Bytes;
use futures::stream::BoxStream;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use object_store::GetResult;
use object_store::GetResultPayload;
use object_store::ObjectMeta;
use object_store::Result;
use opendal::Buffer;
use opendal::Operator;

use reearth_flow_common::uri::Uri;

#[derive(Debug)]
pub struct Storage {
    pub(crate) base_uri: Uri,
    pub(crate) inner: Operator,
}

impl std::fmt::Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenDAL({:?})", self.inner)
    }
}

impl Storage {
    pub fn new(base_uri: Uri, op: Operator) -> Self {
        Self {
            base_uri,
            inner: op,
        }
    }

    pub async fn put(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        let _ = self
            .inner
            .write(p, bytes)
            .await
            .map_err(|err| format_object_store_error(err, p))?;
        Ok(())
    }

    pub async fn create_dir(&self, location: &Path) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        let p = if !p.ends_with('/') {
            format!("{}/", p)
        } else {
            p.to_string()
        };
        self.inner
            .create_dir(p.as_str())
            .await
            .map_err(|err| format_object_store_error(err, p.as_str()))
    }

    pub async fn append(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        let mut w = self
            .inner
            .writer_with(p)
            .append(true)
            .await
            .map_err(|err| format_object_store_error(err, p))?;
        w.write(bytes)
            .await
            .map_err(|err| format_object_store_error(err, p))
    }

    pub async fn get(&self, location: &Path) -> Result<GetResult> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        let meta = self
            .inner
            .stat(p)
            .await
            .map_err(|err| format_object_store_error(err, p))?;

        let meta = ObjectMeta {
            location: object_store::path::Path::parse(p)?,
            last_modified: meta.last_modified().unwrap_or_default(),
            size: meta.content_length() as u64,
            e_tag: meta.etag().map(|x| x.to_string()),
            version: None,
        };
        let r = self
            .inner
            .read(p)
            .await
            .map_err(|err| format_object_store_error(err, p))?;
        Ok(GetResult {
            payload: GetResultPayload::Stream(Box::pin(OpendalReader { inner: r })),
            range: (0..meta.size),
            meta,
            attributes: Default::default(),
        })
    }

    pub async fn exists(&self, location: &Path) -> Result<bool> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        fs::exists(p).map_err(|err| object_store::Error::Generic {
            store: "FileError",
            source: Box::new(err),
        })
    }

    pub async fn get_range(&self, location: &Path, range: Range<usize>) -> Result<Bytes> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        let bs = self
            .inner
            .read_with(p)
            .range(range.start as u64..range.end as u64)
            .await
            .map_err(|err| format_object_store_error(err, p))?;

        Ok(bs.to_bytes())
    }

    pub async fn head(&self, location: &Path) -> Result<ObjectMeta> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        let meta = self
            .inner
            .stat(p)
            .await
            .map_err(|err| format_object_store_error(err, p))?;

        Ok(ObjectMeta {
            location: object_store::path::Path::parse(p)?,
            last_modified: meta.last_modified().unwrap_or_default(),
            size: meta.content_length() as u64,
            e_tag: None,
            version: None,
        })
    }

    pub async fn delete(&self, location: &Path) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        self.inner
            .delete(p)
            .await
            .map_err(|err| format_object_store_error(err, p))?;

        Ok(())
    }

    pub async fn list(
        &self,
        prefix: Option<&Path>,
        recursive: bool,
    ) -> Result<BoxStream<'_, Result<Uri>>> {
        let p = prefix.ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", prefix).into(),
            },
        })?;
        let path =
            p.to_str()
                .map(|v| format!("{}/", v))
                .ok_or(object_store::Error::InvalidPath {
                    source: object_store::path::Error::InvalidPath {
                        path: format!("{:?}", prefix).into(),
                    },
                })?;
        let stream = self
            .inner
            .lister_with(&path)
            .recursive(recursive)
            .await
            .map_err(|err| format_object_store_error(err, &path))?;

        let stream = stream.then(|res| async {
            let entry = res.map_err(|err| format_object_store_error(err, ""))?;
            Ok(Uri::for_test(&format!(
                "{}/{}",
                self.base_uri.protocol().as_str_with_separator(),
                entry.path()
            )))
        });
        Ok(stream.boxed())
    }

    pub async fn list_with_result(
        &self,
        prefix: Option<&Path>,
        recursive: bool,
    ) -> Result<Vec<Uri>> {
        let result = self.list(prefix, recursive).await?;
        let result = result.collect::<Vec<_>>().await;
        Ok(result
            .iter()
            .filter_map(|x| x.as_ref().ok())
            .cloned()
            .collect::<Vec<_>>())
    }

    pub async fn copy(&self, from: &Path, to: &Path) -> Result<()> {
        let from = from.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", from).into(),
            },
        })?;
        let to = to.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", to).into(),
            },
        })?;
        self.inner
            .copy(from.as_ref(), to.as_ref())
            .await
            .map_err(|err| format_object_store_error(err, from))?;
        Ok(())
    }
}

pub(crate) fn format_object_store_error(err: opendal::Error, path: &str) -> object_store::Error {
    use opendal::ErrorKind;
    match err.kind() {
        ErrorKind::NotFound => object_store::Error::NotFound {
            path: path.to_string(),
            source: Box::new(err),
        },
        ErrorKind::Unsupported => object_store::Error::NotSupported {
            source: Box::new(err),
        },
        ErrorKind::AlreadyExists => object_store::Error::AlreadyExists {
            path: path.to_string(),
            source: Box::new(err),
        },
        kind => object_store::Error::Generic {
            store: kind.into_static(),
            source: Box::new(err),
        },
    }
}

struct OpendalReader {
    inner: Buffer,
}

impl Stream for OpendalReader {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner
            .try_poll_next_unpin(cx)
            .map(|x| x)
            .map_err(|e| object_store::Error::Generic {
                store: "IoError",
                source: Box::new(e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opendal::services;
    use reearth_flow_common::uri::Uri;
    use std::path::Path;

    async fn create_test_object_store() -> Storage {
        let op = Operator::new(services::Memory::default()).unwrap().finish();
        let object_store = Storage::new(Uri::for_test("ram://"), op);

        let path = Path::new("/data/test.txt");
        let bytes = Bytes::from_static(b"hello, world!");
        object_store.put(path, bytes).await.unwrap();

        let path = Path::new("/data/nested/test.txt");
        let bytes = Bytes::from_static(b"hello, world! I am nested.");
        object_store.put(path, bytes).await.unwrap();
        object_store
    }

    #[tokio::test]
    async fn test_basic() {
        let op = Operator::new(services::Memory::default()).unwrap().finish();
        let object_store = Storage::new(Uri::for_test("ram://"), op);

        // Retrieve a specific file
        let path = Path::new("/data/test.txt");
        let bytes = Bytes::from_static(b"hello, world!");
        object_store.put(path, bytes.clone()).await.unwrap();

        let meta = object_store.head(path).await.unwrap();

        assert_eq!(meta.size, 13);

        assert_eq!(
            object_store.get(path).await.unwrap().bytes().await.unwrap(),
            bytes
        );
    }

    #[tokio::test]
    async fn test_list() {
        let object_store = create_test_object_store().await;
        let path = Path::new("/data/");
        let results = object_store
            .list(Some(path), false)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        assert_eq!(results.len(), 2);
        let locations = results
            .iter()
            .map(|x| x.as_ref().unwrap())
            .collect::<Vec<_>>();
        let p1 = Uri::for_test("ram:///data/nested/");
        let p2 = Uri::for_test("ram:///data/test.txt");
        let expected_files = vec![&p1, &p2];
        assert_eq!(locations, expected_files);
    }
}
