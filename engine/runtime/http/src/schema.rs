//! Schema fetching for XML validation. Remote schemas are fetched through the
//! egress-guarded client, so a schemaLocation in a document (or an
//! `xs:import` in a fetched XSD) cannot reach internal addresses.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use fastxml::schema::fetcher::error::FetchError;
use fastxml::schema::fetcher::{
    CachingFetcher, CombinedFetcher, FetchResult, FileFetcher, SchemaFetcher,
};
use once_cell::sync::Lazy;

use crate::client::{BlockingHttpClient, HttpClientBuilder};
use crate::error::EgressError;

/// Fetcher for schemaLocation values and the imports they pull in: local
/// files relative to `base_dir`, and http/https through the egress guard.
/// Results are cached in memory and, across processes, on disk.
pub fn schema_fetcher(base_dir: Option<&Path>) -> impl SchemaFetcher {
    let file = match base_dir {
        Some(dir) => FileFetcher::with_base_dir(dir),
        None => FileFetcher::new(),
    };
    let inner = CombinedFetcher::new()
        .with_fetcher(file)
        .with_fetcher(EgressSchemaFetcher);
    CachingFetcher::new(DiskCachingFetcher {
        inner,
        dir: SCHEMA_CACHE_DIR.clone(),
    })
}

/// Created on first use and never dropped, so creating it from inside a
/// tokio runtime is not an issue.
static SCHEMA_HTTP_CLIENT: Lazy<Result<BlockingHttpClient, String>> = Lazy::new(|| {
    HttpClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .max_redirects(Some(10))
        .build_blocking()
        .map_err(|e| e.to_string())
});

struct EgressSchemaFetcher;

impl SchemaFetcher for EgressSchemaFetcher {
    fn fetch(&self, url: &str) -> fastxml::error::Result<FetchResult> {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(FetchError::RequestFailed {
                url: url.to_string(),
                message: "not an http or https URL".to_string(),
            }
            .into());
        }
        let client =
            SCHEMA_HTTP_CLIENT
                .as_ref()
                .map_err(|message| FetchError::ClientCreationFailed {
                    message: message.clone(),
                })?;
        let request_failed = |e: EgressError| FetchError::RequestFailed {
            url: url.to_string(),
            message: e.to_string(),
        };

        let requested: Option<url::Url> = url.parse().ok();
        let response = client
            .get(url)
            .map_err(request_failed)?
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
        let content = response
            .bytes()
            .map_err(|e| FetchError::ReadResponseFailed {
                message: e.to_string(),
            })?
            .to_vec();
        Ok(FetchResult {
            content,
            final_url,
            redirected,
        })
    }
}

/// Persistent on-disk cache directory for fetched XSD schemas.
///
/// Remote GML/CityGML base schemas (e.g. `http://schemas.opengis.net/...`) are
/// referenced by absolute URL and are not shipped in the local schema bundle, so
/// without a persistent cache every validation run re-downloads them. Pointing the
/// `FileCachingFetcher` at a stable directory makes those downloads happen once and
/// be reused across documents, workflow runs, and processes.
///
/// Override the location with `FLOW_RUNTIME_XML_SCHEMA_CACHE_DIR`.
static SCHEMA_CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = std::env::var("FLOW_RUNTIME_XML_SCHEMA_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("reearth-flow-xml-schema-cache"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!(dir = ?dir, error = ?e, "Failed to create XML schema cache directory");
    }
    dir
});

/// Per-URL locks used to coalesce concurrent fetches of the same schema URL
/// within a process ("single-flight").
///
/// On a cold runner the on-disk cache is empty, so every worker thread that
/// starts a workflow at once races to download the same handful of remote
/// XSDs (the CityGML/GML base schemas). Sixteen threads hitting
/// `schemas.opengis.net` simultaneously trips its rate limiter and turns a
/// few-second download into minutes of retries. Electing a single thread per
/// URL to do the fetch — while the rest wait and then read the file it wrote —
/// removes the herd without touching the warm-cache fast path.
///
/// The map only ever grows by the number of distinct schema URLs seen in a
/// process (a few dozen), so it is not a meaningful leak.
static FETCH_LOCKS: Lazy<parking_lot::Mutex<HashMap<String, Arc<parking_lot::Mutex<()>>>>> =
    Lazy::new(|| parking_lot::Mutex::new(HashMap::new()));

fn fetch_lock_for(url: &str) -> Arc<parking_lot::Mutex<()>> {
    let mut locks = FETCH_LOCKS.lock();
    Arc::clone(
        locks
            .entry(url.to_string())
            .or_insert_with(|| Arc::new(parking_lot::Mutex::new(()))),
    )
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
struct DiskCachingFetcher<F> {
    inner: F,
    dir: PathBuf,
}

impl<F> DiskCachingFetcher<F> {
    fn cache_path(&self, url: &str) -> PathBuf {
        use std::hash::{Hash, Hasher};
        // DefaultHasher uses fixed keys (not randomized), so the filename is
        // stable across processes and machines.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        self.dir.join(format!("{:016x}.xsd", hasher.finish()))
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

        // Cache miss: coalesce concurrent fetches of this URL (see FETCH_LOCKS).
        let url_lock = fetch_lock_for(url);
        let _guard = url_lock.lock();

        // Re-check: the elected thread may have populated the cache while we
        // waited for the lock.
        if let Some(result) = read_cached_schema(&path, url) {
            return Ok(result);
        }

        let result = fetch_uncached()?;

        // Only cache direct (non-redirected) responses so a cached entry's
        // `final_url` stays correct for relative-import resolution.
        if !result.redirected && result.final_url == url {
            write_cached_schema(&path, &result.content);
        }

        Ok(result)
    }
}

impl<F: SchemaFetcher> SchemaFetcher for DiskCachingFetcher<F> {
    fn fetch(&self, url: &str) -> fastxml::error::Result<FetchResult> {
        self.fetch_with_cache(url, || self.inner.fetch(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_caching_fetcher_coalesces_concurrent_fetches() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Barrier,
        };

        // Each thread fetches the same URL through a temp-dir-backed disk cache.
        // The "network" fetch is a counting closure, so we can assert that the
        // single-flight lock collapses the herd into exactly one real fetch
        // (the rest are served from the file that fetch wrote). This is the
        // cold-runner scenario that previously stampeded schemas.opengis.net.
        let tmp = tempfile::tempdir().expect("create temp cache dir");
        let fetcher = Arc::new(DiskCachingFetcher {
            inner: fastxml::schema::fetcher::NoopFetcher,
            dir: tmp.path().to_path_buf(),
        });

        // Unique URL so neither the process-global FETCH_LOCKS map nor the
        // per-test temp dir can be warmed by another test.
        let url = "http://example.test/coalesce/unique-schema-aa17.xsd";
        let network_calls = Arc::new(AtomicUsize::new(0));

        const THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(THREADS));

        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let fetcher = Arc::clone(&fetcher);
                let network_calls = Arc::clone(&network_calls);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
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

        let results: Vec<FetchResult> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(
            network_calls.load(Ordering::SeqCst),
            1,
            "concurrent fetches of the same URL must coalesce into one network call"
        );
        for result in results {
            assert_eq!(
                result.content,
                b"<xsd:schema/>".to_vec(),
                "every caller must receive the fetched content"
            );
        }
    }
}
