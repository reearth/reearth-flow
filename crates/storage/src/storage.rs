use std::ops::Range;
use std::path::Path;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use bytes::Bytes;
use futures::stream::BoxStream;
use futures::Stream;
use futures::StreamExt;
use object_store::GetResult;
use object_store::GetResultPayload;
use object_store::ListResult;
use object_store::ObjectMeta;
use object_store::Result;
use opendal::Metadata;
use opendal::Metakey;
use opendal::Operator;
use opendal::Reader;

#[derive(Debug)]
pub struct Storage {
    inner: Operator,
}

impl std::fmt::Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenDAL({:?})", self.inner)
    }
}

impl Storage {
    pub fn new(op: Operator) -> Self {
        Self { inner: op }
    }

    pub async fn put(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let p = location.to_str().ok_or(object_store::Error::InvalidPath {
            source: object_store::path::Error::InvalidPath {
                path: format!("{:?}", location).into(),
            },
        })?;
        self.inner
            .write(p, bytes)
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
            size: meta.content_length() as usize,
            e_tag: meta.etag().map(|x| x.to_string()),
            version: None,
        };
        let r = self
            .inner
            .reader(p)
            .await
            .map_err(|err| format_object_store_error(err, p))?;

        Ok(GetResult {
            payload: GetResultPayload::Stream(Box::pin(OpendalReader { inner: r })),
            range: (0..meta.size),
            meta,
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

        Ok(Bytes::from(bs))
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
            size: meta.content_length() as usize,
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

    pub async fn list(&self, prefix: Option<&Path>) -> Result<BoxStream<'_, Result<ObjectMeta>>> {
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
            .recursive(false)
            .metakey(Metakey::ContentLength | Metakey::LastModified)
            .await
            .map_err(|err| format_object_store_error(err, &path))?;

        let stream = stream.then(|res| async {
            let entry = res.map_err(|err| format_object_store_error(err, ""))?;
            let meta = entry.metadata();

            Ok(format_object_meta(entry.path(), meta))
        });

        Ok(stream.boxed())
    }

    pub async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
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
        let mut stream = self
            .inner
            .lister_with(&path)
            .metakey(Metakey::Mode | Metakey::ContentLength | Metakey::LastModified)
            .await
            .map_err(|err| format_object_store_error(err, &path))?;

        let mut common_prefixes = Vec::new();
        let mut objects = Vec::new();

        while let Some(res) = stream.next().await {
            let entry = res.map_err(|err| format_object_store_error(err, ""))?;
            let meta = entry.metadata();

            if meta.is_dir() {
                common_prefixes.push(entry.path().into());
            } else {
                objects.push(format_object_meta(entry.path(), meta));
            }
        }

        Ok(ListResult {
            common_prefixes,
            objects,
        })
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

fn format_object_store_error(err: opendal::Error, path: &str) -> object_store::Error {
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

fn format_object_meta(path: &str, meta: &Metadata) -> ObjectMeta {
    ObjectMeta {
        location: path.into(),
        last_modified: meta.last_modified().unwrap_or_default(),
        size: meta.content_length() as usize,
        e_tag: None,
        version: None,
    }
}

struct OpendalReader {
    inner: Reader,
}

impl Stream for OpendalReader {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use opendal::raw::oio::Read;

        self.inner
            .poll_next(cx)
            .map_err(|err| object_store::Error::Generic {
                store: "IoError",
                source: Box::new(err),
            })
    }
}

#[cfg(test)]
mod tests {
    use opendal::services;
    use std::path::Path;

    use super::*;

    async fn create_test_object_store() -> Storage {
        let op = Operator::new(services::Memory::default()).unwrap().finish();
        let object_store = Storage::new(op);

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
        let object_store = Storage::new(op);

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
            .list(Some(path))
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        assert_eq!(results.len(), 2);
        let mut locations = results
            .iter()
            .map(|x| x.as_ref().unwrap().location.as_ref())
            .collect::<Vec<_>>();

        let expected_files = vec!["data/nested", "data/test.txt"];
        locations.sort();
        assert_eq!(locations, expected_files);
    }

    #[tokio::test]
    async fn test_list_with_delimiter() {
        let object_store = create_test_object_store().await;
        let path = Path::new("/data/");
        let result = object_store.list_with_delimiter(Some(path)).await.unwrap();
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.common_prefixes.len(), 1);
        assert_eq!(result.objects[0].location.as_ref(), "data/test.txt");
        assert_eq!(result.common_prefixes[0].as_ref(), "data/nested");
    }
}
