//! Schema fetching for XML validation. Remote schemas are fetched through the
//! egress-guarded client, so a schemaLocation in a document (or an
//! `xs:import` in a fetched XSD) cannot reach internal addresses.

use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::body::read_to_end_capped;
use crate::client::BlockingHttpClient;
use crate::error::EgressError;
use fastxml::schema::fetcher::error::FetchError;
use fastxml::schema::fetcher::{
    CachingFetcher, CombinedFetcher, FetchResult, FileFetcher, SchemaFetcher,
};

/// Overrides the directory of the on-disk schema cache.
pub const SCHEMA_CACHE_DIR_ENV: &str = "FLOW_RUNTIME_XML_SCHEMA_CACHE_DIR";

/// Limit on a fetched schema, both as received and after gzip decoding. Real
/// XSDs are at most a few hundred KB.
const MAX_SCHEMA_SIZE: u64 = 16 * 1024 * 1024;

/// Persistent on-disk cache for fetched XSD schemas.
///
/// Remote GML/CityGML base schemas (e.g. `http://schemas.opengis.net/...`) are
/// referenced by absolute URL and are not shipped in the local schema bundle,
/// so without a persistent cache every validation run re-downloads them.
///
/// Cached files are shared by every instance using the same directory, in any
/// process. Concurrent fetches of the same URL are coalesced only within one
/// instance, so keep a single instance per process.
pub struct SchemaDiskCache {
    dir: PathBuf,
    /// Per-URL locks that coalesce concurrent fetches of the same schema URL
    /// ("single-flight").
    ///
    /// On a cold runner the on-disk cache is empty, so every worker thread
    /// that starts at once races to download the same handful of remote XSDs
    /// (the CityGML/GML base schemas). Sixteen threads hitting
    /// `schemas.opengis.net` simultaneously trips its rate limiter and turns a
    /// few-second download into minutes of retries. Electing a single thread
    /// per URL to do the fetch — while the rest wait and then read the file it
    /// wrote — removes the herd without touching the warm-cache fast path.
    ///
    /// An entry is removed once no caller holds it, so the map only holds URLs
    /// being fetched.
    fetch_locks: parking_lot::Mutex<HashMap<String, Arc<parking_lot::Mutex<()>>>>,
}

impl SchemaDiskCache {
    pub fn new(dir: PathBuf) -> Self {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::warn!(dir = ?dir, error = ?e, "Failed to create XML schema cache directory");
        }
        Self {
            dir,
            fetch_locks: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    /// Cache in the directory named by `FLOW_RUNTIME_XML_SCHEMA_CACHE_DIR`, or
    /// in a fixed directory under the system temp dir. Every instance created
    /// this way shares the same cached files, across workflow runs and
    /// processes.
    pub fn from_env() -> Self {
        let dir = std::env::var_os(SCHEMA_CACHE_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("reearth-flow-xml-schema-cache"));
        Self::new(dir)
    }

    /// Fetcher for schemaLocation values and the imports they pull in: local
    /// files relative to `base_dir`, and http/https through `client`. Results
    /// are cached in memory and on disk.
    pub fn fetcher<'a>(
        &'a self,
        client: &BlockingHttpClient,
        base_dir: Option<&Path>,
    ) -> impl SchemaFetcher + 'a {
        let file = match base_dir {
            Some(dir) => FileFetcher::with_base_dir(dir),
            None => FileFetcher::new(),
        };
        let inner = CombinedFetcher::new()
            .with_fetcher(file)
            .with_fetcher(EgressSchemaFetcher {
                client: client.clone(),
            });
        CachingFetcher::new(DiskCachingFetcher { inner, cache: self })
    }

    fn fetch_lock_for(&self, url: &str) -> Arc<parking_lot::Mutex<()>> {
        let mut locks = self.fetch_locks.lock();
        Arc::clone(
            locks
                .entry(url.to_string())
                .or_insert_with(|| Arc::new(parking_lot::Mutex::new(()))),
        )
    }

    /// Drop `url`'s entry unless another caller still holds or waits on it.
    fn release_fetch_lock(&self, url: &str, lock: Arc<parking_lot::Mutex<()>>) {
        let mut locks = self.fetch_locks.lock();
        // Only the map and `lock` remain, and new callers need the map lock.
        if Arc::strong_count(&lock) == 2 {
            locks.remove(url);
        }
    }
}

struct EgressSchemaFetcher {
    /// A clone shares the connection pool of the client it was cloned from.
    client: BlockingHttpClient,
}

impl SchemaFetcher for EgressSchemaFetcher {
    fn fetch(&self, url: &str) -> fastxml::error::Result<FetchResult> {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(FetchError::RequestFailed {
                url: url.to_string(),
                message: "not an http or https URL".to_string(),
            }
            .into());
        }
        let request_failed = |e: EgressError| FetchError::RequestFailed {
            url: url.to_string(),
            message: e.to_string(),
        };

        let requested: Option<url::Url> = url.parse().ok();
        let mut response = self
            .client
            .get(url)
            .map_err(request_failed)?
            .header(reqwest::header::ACCEPT_ENCODING, "gzip")
            .send()
            .map_err(|e| request_failed(EgressError::Request(e)))?;

        let status = response.status().as_u16();
        if status != 200 {
            return Err(FetchError::HttpError {
                status,
                url: url.to_string(),
            }
            .into());
        }
        // Compare parsed URLs so normalization (e.g. a trailing slash) is not
        // mistaken for a redirect.
        let redirected = requested.as_ref() != Some(response.url());
        let final_url = if redirected {
            response.url().to_string()
        } else {
            url.to_string()
        };
        let gzipped = response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| {
                let v = v.trim();
                v.eq_ignore_ascii_case("gzip") || v.eq_ignore_ascii_case("x-gzip")
            });
        let read_failed = |e: &dyn std::fmt::Display| FetchError::ReadResponseFailed {
            message: e.to_string(),
        };
        let read_capped = |reader: &mut dyn Read| {
            read_to_end_capped(reader, MAX_SCHEMA_SIZE)
                .map_err(|e| read_failed(&e))?
                .ok_or_else(|| read_failed(&format!("schema exceeds {MAX_SCHEMA_SIZE} bytes")))
        };
        let body = read_capped(&mut response)?;
        // Decoded here because enabling reqwest's `gzip` feature would change every
        // reqwest client in the workspace.
        let content = if gzipped {
            read_capped(&mut flate2::read::MultiGzDecoder::new(&body[..]))?
        } else {
            body
        };
        Ok(FetchResult {
            content,
            final_url,
            redirected,
        })
    }
}

/// Read a previously cached schema from disk, if present.
///
/// Only a regular file is served: the cache directory lives under a shared
/// temp dir, so a symlink planted at the cache path must not be followed to
/// read arbitrary content elsewhere on the filesystem.
fn read_cached_schema(path: &Path, url: &str) -> Option<FetchResult> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.file_type().is_file() {
        return None;
    }
    let content = std::fs::read(path).ok()?;
    Some(FetchResult {
        content,
        final_url: url.to_string(),
        redirected: false,
    })
}

/// Atomically write a fetched schema to the disk cache.
///
/// Writing via a sibling temp file + rename means a pre-existing symlink at the
/// destination is not followed, and concurrent readers see either the old or
/// the new content, never a partial write.
fn write_cached_schema(path: &Path, content: &[u8]) {
    let Some(parent) = path.parent() else {
        return;
    };
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("cache"),
        std::process::id()
    ));
    if std::fs::write(&tmp, content).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

/// Persistent, cross-process disk cache for fetched XSD schemas.
///
/// `fastxml`'s own `FileCachingFetcher` only consults its in-memory index, which
/// starts empty on every process, so its on-disk files are never re-read across
/// runs. This wrapper keys directly off a deterministic filename: a fresh process
/// finds the existing file on disk and skips the network entirely. Concurrent
/// misses for the same URL are coalesced into a single network fetch.
struct DiskCachingFetcher<'a, F> {
    inner: F,
    cache: &'a SchemaDiskCache,
}

impl<F> DiskCachingFetcher<'_, F> {
    fn cache_path(&self, url: &str) -> PathBuf {
        use std::hash::{Hash, Hasher};
        // DefaultHasher uses fixed keys (not randomized), so the filename is
        // stable across processes and machines.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        self.cache.dir.join(format!("{:016x}.xsd", hasher.finish()))
    }

    /// Serve `url` from the disk cache, or fetch it via `fetch_uncached` and
    /// populate the cache. Concurrent callers for the same URL are serialized
    /// so only one runs `fetch_uncached`; the rest read the file it wrote.
    fn fetch_with_cache(
        &self,
        url: &str,
        fetch_uncached: impl FnOnce() -> fastxml::error::Result<FetchResult>,
    ) -> fastxml::error::Result<FetchResult> {
        let path = self.cache_path(url);

        // Fast path: serve directly from disk without taking any lock.
        if let Some(result) = read_cached_schema(&path, url) {
            return Ok(result);
        }

        // Cache miss: coalesce concurrent fetches of this URL (see `SchemaDiskCache::fetch_locks`).
        let url_lock = self.cache.fetch_lock_for(url);
        let result = {
            let _guard = url_lock.lock();

            // Re-check: the elected thread may have populated the cache while we
            // waited for the lock.
            match read_cached_schema(&path, url) {
                Some(result) => Ok(result),
                None => fetch_uncached().inspect(|result| {
                    // Only cache direct (non-redirected) responses so a cached entry's
                    // `final_url` stays correct for relative-import resolution.
                    if !result.redirected && result.final_url == url {
                        write_cached_schema(&path, &result.content);
                    }
                }),
            }
        };
        self.cache.release_fetch_lock(url, url_lock);
        result
    }
}

impl<F: SchemaFetcher> SchemaFetcher for DiskCachingFetcher<'_, F> {
    fn fetch(&self, url: &str) -> fastxml::error::Result<FetchResult> {
        self.fetch_with_cache(url, || self.inner.fetch(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::HttpClientBuilder;

    #[test]
    fn test_disk_caching_fetcher_coalesces_concurrent_fetches() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Barrier,
        };

        // Each thread fetches the same URL through a temp-dir-backed disk cache.
        // The "network" fetch is a counting closure, so we can assert that the
        // single-flight lock collapses the herd into exactly one real fetch
        // (the rest are served from the file that fetch wrote). This is the
        // cold-runner scenario that previously stampeded schemas.opengis.net.
        let tmp = tempfile::tempdir().expect("create temp cache dir");
        let cache = SchemaDiskCache::new(tmp.path().to_path_buf());
        let fetcher = DiskCachingFetcher {
            inner: fastxml::schema::fetcher::NoopFetcher,
            cache: &cache,
        };

        let url = "http://example.test/coalesce/schema.xsd";
        let network_calls = AtomicUsize::new(0);

        const THREADS: usize = 16;
        let barrier = Barrier::new(THREADS);

        let results: Vec<FetchResult> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..THREADS)
                .map(|_| {
                    s.spawn(|| {
                        // Release all threads together so they genuinely contend.
                        barrier.wait();
                        fetcher
                            .fetch_with_cache(url, || {
                                network_calls.fetch_add(1, Ordering::SeqCst);
                                // Widen the race window so the late threads are
                                // forced through the lock + disk re-check path.
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                Ok(FetchResult {
                                    content: b"<xsd:schema/>".to_vec(),
                                    final_url: url.to_string(),
                                    redirected: false,
                                })
                            })
                            .expect("fetch_with_cache should succeed")
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        assert_eq!(
            network_calls.load(Ordering::SeqCst),
            1,
            "concurrent fetches of the same URL must coalesce into one network call"
        );
        assert!(
            cache.fetch_locks.lock().is_empty(),
            "finished fetches must not leave lock entries behind"
        );
        for result in results {
            assert_eq!(
                result.content,
                b"<xsd:schema/>".to_vec(),
                "every caller must receive the fetched content"
            );
        }
    }

    /// Serves `content` gzip-encoded once on loopback and fetches it. Returns
    /// the fetch result and the request the server received.
    fn fetch_gzipped(content: &[u8]) -> (fastxml::error::Result<FetchResult>, String) {
        use std::io::Write;
        use std::net::TcpListener;

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(content).unwrap();
        let gzipped = encoder.finish().unwrap();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/schema.xsd", server.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = server.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase();
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\
                 Connection: close\r\n\r\n",
                gzipped.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(&gzipped).unwrap();
            request
        });
        // Private networking is allowed so the test server on loopback is
        // reachable.
        let fetcher = EgressSchemaFetcher {
            client: HttpClientBuilder::new()
                .policy(crate::policy::EgressPolicy {
                    allow_private_network: true,
                })
                .build_blocking()
                .unwrap(),
        };

        let result = fetcher.fetch(&url);
        (result, handle.join().unwrap())
    }

    #[test]
    fn test_egress_schema_fetcher_decodes_gzip_response() {
        let schema = b"<xsd:schema xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\"/>";

        let (result, request) = fetch_gzipped(schema);

        assert!(request.contains("accept-encoding: gzip"), "{request}");
        assert_eq!(
            result.expect("fetch should succeed").content,
            schema.to_vec()
        );
    }

    #[test]
    fn test_egress_schema_fetcher_rejects_oversized_decoded_schema() {
        let content = vec![b' '; MAX_SCHEMA_SIZE as usize + 1];

        let (result, _) = fetch_gzipped(&content);

        let err = result.expect_err("oversized schema must be rejected");
        assert!(err.to_string().contains("exceeds"), "{err}");
    }
}
