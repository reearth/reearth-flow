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
use tracing::info;
use tracing::warn;

/// Chunk size for [`Storage::put`]. GCS multipart parts must be at least 5 MiB.
const WRITE_CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Maximum number of chunks in flight per [`Storage::put`].
const WRITE_CONCURRENCY: usize = 4;

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

    /// Writes `bytes` to `location` in chunks of [`WRITE_CHUNK_SIZE`].
    ///
    /// Each chunk is a separate IO call, so the operator's IO timeout applies
    /// per chunk rather than to the whole object. On object stores, data larger
    /// than one chunk goes up as a multipart upload, which is aborted on failure.
    pub async fn put(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let mut w = self
            .inner
            .writer_with(p)
            .chunk(WRITE_CHUNK_SIZE)
            .concurrent(WRITE_CONCURRENCY)
            .await
            .map_err(|err| format_object_store_error(err, p))?;
        let result = match w.write(bytes).await {
            Ok(()) => w.close().await.map(|_| ()),
            Err(err) => Err(err),
        };
        if let Err(err) = result {
            // Discard uploaded parts. A no-op if no multipart upload was started.
            if let Err(abort_err) = w.abort().await {
                // Fs without an atomic write dir cannot abort.
                if abort_err.kind() != opendal::ErrorKind::Unsupported {
                    warn!("Failed to abort write to {p}: {abort_err}");
                }
            }
            return Err(format_object_store_error(err, p));
        }
        Ok(())
    }

    /// Like [`Storage::put`], but an existing object of the same size counts as
    /// already written.
    ///
    /// Checked before writing, to skip objects that a rerun already uploaded,
    /// and after a failed write, because a write can land even when its
    /// response is lost and the retry then fails (e.g. overwrite not permitted).
    /// Only use it for paths owned by a single writer, since any object of the
    /// same size is accepted.
    pub async fn put_idempotent(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
            },
        })?;
        let size = bytes.len() as u64;
        if self.has_object_with_size(p, size).await? {
            info!("Object {p} already exists with the expected size; skipping write");
            return Ok(());
        }
        match self.put(location, bytes).await {
            Ok(()) => Ok(()),
            Err(err) => {
                if self.has_object_with_size(p, size).await.unwrap_or(false) {
                    warn!(
                        "Write to {p} reported an error, but the object exists with the expected size; treating as written: {err}"
                    );
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn has_object_with_size(&self, p: &str, size: u64) -> Result<bool> {
        match self.inner.stat(p).await {
            Ok(meta) => Ok(meta.content_length() == size),
            Err(err) if err.kind() == opendal::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(format_object_store_error(err, p)),
        }
    }

    pub async fn create_dir(&self, location: &Path) -> Result<()> {
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
            .create_dir(p.as_str())
            .await
            .map_err(|err| format_object_store_error(err, p.as_str()))
    }

    pub async fn append(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
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
            .map_err(|err| format_object_store_error(err, p))?;
        w.close()
            .await
            .map_err(|err| format_object_store_error(err, p))?;
        Ok(())
    }

    pub async fn get(&self, location: &Path) -> Result<GetResult> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
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
            size: meta.content_length(),
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
            extensions: Default::default(),
        })
    }

    pub async fn exists(&self, location: &Path) -> Result<bool> {
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

    pub async fn get_range(&self, location: &Path, range: Range<usize>) -> Result<Bytes> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
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
                path: format!("{location:?}").into(),
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
            size: meta.content_length(),
            e_tag: None,
            version: None,
        })
    }

    pub async fn delete(&self, location: &Path) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{location:?}").into(),
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
                path: format!("{from:?}").into(),
            },
        })?;
        let to = to.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{to:?}").into(),
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

    fn memory_store() -> Storage {
        let op = Operator::new(services::Memory::default()).unwrap().finish();
        Storage::new(Uri::for_test("ram://"), op)
    }

    async fn read_all(store: &Storage, path: &Path) -> Bytes {
        store.get(path).await.unwrap().bytes().await.unwrap()
    }

    #[tokio::test]
    async fn test_put_larger_than_chunk() {
        let store = memory_store();
        let path = Path::new("/data/large.bin");
        let bytes: Bytes = (0..(2 * WRITE_CHUNK_SIZE + 123))
            .map(|i| (i % 251) as u8)
            .collect::<Vec<_>>()
            .into();
        store.put(path, bytes.clone()).await.unwrap();
        assert_eq!(read_all(&store, path).await, bytes);
    }

    #[tokio::test]
    async fn test_put_idempotent_writes_missing_object() {
        let store = memory_store();
        let path = Path::new("/data/new.txt");
        store
            .put_idempotent(path, Bytes::from_static(b"hello"))
            .await
            .unwrap();
        assert_eq!(read_all(&store, path).await, Bytes::from_static(b"hello"));
    }

    #[tokio::test]
    async fn test_put_idempotent_skips_object_with_same_size() {
        let store = memory_store();
        let path = Path::new("/data/existing.txt");
        store.put(path, Bytes::from_static(b"first")).await.unwrap();
        store
            .put_idempotent(path, Bytes::from_static(b"other"))
            .await
            .unwrap();
        assert_eq!(read_all(&store, path).await, Bytes::from_static(b"first"));
    }

    #[tokio::test]
    async fn test_put_idempotent_writes_when_size_differs() {
        let store = memory_store();
        let path = Path::new("/data/existing.txt");
        store.put(path, Bytes::from_static(b"short")).await.unwrap();
        store
            .put_idempotent(path, Bytes::from_static(b"much longer"))
            .await
            .unwrap();
        assert_eq!(
            read_all(&store, path).await,
            Bytes::from_static(b"much longer")
        );
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

    #[test]
    fn test_stream_to_file_sync() {
        use std::io::{Read, Seek, SeekFrom};

        let op = Operator::new(services::Memory::default()).unwrap().finish();
        let storage = Storage::new(Uri::for_test("ram://"), op);

        let data = b"hello, world!";
        storage
            .put_sync(Path::new("/data/test.bin"), Bytes::from_static(data))
            .unwrap();

        let mut tmp = tempfile::tempfile().unwrap();
        storage
            .stream_to_file_sync(Path::new("/data/test.bin"), &mut tmp)
            .unwrap();

        tmp.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = Vec::new();
        tmp.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, data);
    }
}
