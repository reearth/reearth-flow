use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Write;
use std::{
    borrow::Cow,
    env,
    io::{Error, Result},
    path::{Path, PathBuf},
    sync::Arc,
};

use reearth_flow_common::str::remove_trailing_slash;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;

const CHUNK_SIZE: usize = 1000;

const ZSTD_LEVEL: i32 = 1;

const MAX_DIRECTORY_DEPTH: usize = 64;

const MAX_HITS: usize = 2;

#[derive(Debug, Clone)]
pub struct State {
    storage: Arc<Storage>,
    root: PathBuf,
    use_compression: bool,
}

impl State {
    pub fn new(root: &Uri, storage_resolver: &StorageResolver) -> Result<Self> {
        let storage = storage_resolver
            .resolve(root)
            .map_err(std::io::Error::other)?;
        let use_compression = env::var("FLOW_RUNTIME_COMPRESS_INTERMEDIATE_DATA")
            .ok()
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);
        Ok(Self {
            storage,
            root: Path::new(
                remove_trailing_slash(root.path().to_str().unwrap_or_default()).as_str(),
            )
            .to_path_buf(),
            use_compression,
        })
    }

    pub async fn save<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.json_ext());
        self.storage
            .put(p.as_path(), content)
            .await
            .map_err(Error::other)
    }

    pub fn save_sync<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.json_ext());
        self.storage
            .put_sync(p.as_path(), content)
            .map_err(Error::other)
    }

    pub async fn append<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)? + "\n";
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.jsonl_ext());
        self.storage
            .append(p.as_path(), content)
            .await
            .map_err(Error::other)
    }

    pub async fn append_strings(&self, all: &[String], id: &str) -> Result<()> {
        if all.is_empty() {
            return Ok(());
        }
        let p = self.id_to_location(id, self.jsonl_ext());
        for chunk in all.chunks(CHUNK_SIZE) {
            let s = chunk.join("\n") + "\n";
            let content = self.encode(s.as_bytes())?;
            self.storage
                .append(p.as_path(), content)
                .await
                .map_err(Error::other)?
        }
        Ok(())
    }

    pub fn append_sync<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)? + "\n";
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.jsonl_ext());
        self.storage
            .append_sync(p.as_path(), content)
            .map_err(Error::other)
    }

    pub async fn get<T>(&self, id: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let result = self
            .storage
            .get(self.id_to_location(id, self.json_ext()).as_path())
            .await?;
        let bytes = result.bytes().await?;
        let data = self.decode(bytes.as_ref())?;
        let s = std::str::from_utf8(&data).map_err(Error::other)?;
        self.string_to_object::<T>(s)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.storage
            .delete(self.id_to_location(id, self.json_ext()).as_path())
            .await
            .map_err(Error::other)
    }

    pub fn id_to_location(&self, id: &str, ext: &str) -> PathBuf {
        PathBuf::new()
            .join(self.root.clone())
            .join(format!("{id}.{ext}"))
    }

    pub fn string_to_object<T>(&self, s: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        serde_json::from_str(s).map_err(Error::other)
    }

    pub fn object_to_string<T: Serialize>(&self, obj: &T) -> Result<String> {
        serde_json::to_string(obj).map_err(Error::other)
    }

    fn json_ext(&self) -> &'static str {
        if self.use_compression {
            "json.zst"
        } else {
            "json"
        }
    }

    fn jsonl_ext(&self) -> &'static str {
        if self.use_compression {
            "jsonl.zst"
        } else {
            "jsonl"
        }
    }

    fn encode(&self, bytes: &[u8]) -> Result<bytes::Bytes> {
        if self.use_compression {
            let compressed = zstd::stream::encode_all(bytes, ZSTD_LEVEL).map_err(Error::other)?;
            Ok(bytes::Bytes::from(compressed))
        } else {
            Ok(bytes::Bytes::copy_from_slice(bytes))
        }
    }

    fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, [u8]>> {
        if self.use_compression {
            let v = zstd::stream::decode_all(bytes).map_err(Error::other)?;
            Ok(Cow::Owned(v))
        } else {
            Ok(Cow::Borrowed(bytes))
        }
    }

    /// Copies a JSONL/JSONL.zst file identified by `id` from `src` into `self` (blocking, CLI-friendly).
    pub fn copy_jsonl_from_state(&self, src: &State, id: &str) -> Result<()> {
        // Try direct match first (e.g., "edge-id" or "workflow-id.edge-id")
        if let Ok(()) = self.copy_jsonl_from_state_exact(src, id) {
            return Ok(());
        }

        if id.contains('.') {
            return Err(Error::other("no candidate jsonl found"));
        }

        // Scan directory for namespaced files matching "<something>.<edge_id>.jsonl"
        let edge_id = id;
        let resolved_ids = src.find_namespaced_jsonl_ids_sync(edge_id);

        tracing::info!(
            "Resolved jsonl ids for edge_id={edge_id}: {:?}",
            resolved_ids
        );

        if resolved_ids.is_empty() {
            return Err(Error::other("no candidate jsonl found"));
        }
        if resolved_ids.len() > 1 {
            return Err(Error::other(format!(
                "multiple candidate jsonl found for edge_id={edge_id}: {:?}",
                resolved_ids
            )));
        }

        // Copy using the resolved namespaced id
        self.copy_jsonl_from_state_exact(src, &resolved_ids[0])
    }

    /// Copies a JSONL/JSONL.zst file identified by `id` from `src` into `self` (non-blocking, worker-friendly).
    pub async fn copy_jsonl_from_state_async(&self, src: &State, id: &str) -> Result<()> {
        // Try direct match first (e.g., "edge-id" or "workflow-id.edge-id")
        if let Ok(()) = self.copy_jsonl_from_state_exact_async(src, id).await {
            return Ok(());
        }

        if id.contains('.') {
            return Err(Error::other("no candidate jsonl found"));
        }

        // Scan directory for namespaced files matching "<something>.<edge_id>.jsonl"
        let edge_id = id;
        let resolved_ids = src.find_namespaced_jsonl_ids_async(edge_id).await;

        tracing::info!(
            "Resolved jsonl ids for edge_id={edge_id}: {:?}",
            resolved_ids
        );

        if resolved_ids.is_empty() {
            return Err(Error::other("no candidate jsonl found"));
        }
        if resolved_ids.len() > 1 {
            return Err(Error::other(format!(
                "multiple candidate jsonl found for edge_id={edge_id}: {:?}",
                resolved_ids
            )));
        }

        // Copy using the resolved namespaced id
        self.copy_jsonl_from_state_exact_async(src, &resolved_ids[0])
            .await
    }

    /// Attempts to copy a JSONL file with exact `id` match.
    fn copy_jsonl_from_state_exact(&self, src: &State, id: &str) -> Result<()> {
        let candidates = if src.use_compression {
            ["jsonl.zst", "jsonl"]
        } else {
            ["jsonl", "jsonl.zst"]
        };

        let mut last_err: Option<Error> = None;

        for ext in candidates {
            let src_path = src.id_to_location(id, ext);
            match src.storage.get_sync(src_path.as_path()) {
                Ok(bytes) => {
                    let dst_path = self.id_to_location(id, ext);
                    return self
                        .storage
                        .put_sync(dst_path.as_path(), bytes)
                        .map_err(Error::other);
                }
                Err(e) => {
                    last_err = Some(Error::other(e));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| Error::other("no candidate jsonl found")))
    }

    /// Copies a JSONL/JSONL.zst file identified by `id` from `src` into `self` (non-blocking, worker-friendly).
    async fn copy_jsonl_from_state_exact_async(&self, src: &State, id: &str) -> Result<()> {
        let candidates = if src.use_compression {
            ["jsonl.zst", "jsonl"]
        } else {
            ["jsonl", "jsonl.zst"]
        };

        let mut last_err: Option<Error> = None;

        for ext in candidates {
            let src_path = src.id_to_location(id, ext);
            match src.storage.get(src_path.as_path()).await {
                Ok(obj) => {
                    let bytes = obj.bytes().await.map_err(Error::other)?;
                    // keep the same extension as the source we actually found
                    let dst_path = self.id_to_location(id, ext);
                    return self
                        .storage
                        .put(dst_path.as_path(), bytes)
                        .await
                        .map_err(Error::other);
                }
                Err(e) => {
                    last_err = Some(Error::other(e));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| Error::other("no candidate jsonl found")))
    }

    /// Scans the local filesystem for JSONL files matching the pattern `*.<edge_id>.jsonl(.zst)`.
    /// Returns a list of resolved file stems (without extensions) that end with `.<edge_id>`.
    ///
    /// This is used for incremental runs with subworkflows where edge files are namespaced
    /// as `<workflow_id>.<edge_id>.jsonl` instead of just `<edge_id>.jsonl`.
    fn find_namespaced_jsonl_ids_sync(&self, edge_id: &str) -> Vec<String> {
        let mut out = Vec::new();

        let Ok(rd) = std::fs::read_dir(&self.root) else {
            return out;
        };

        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            let stem = if let Some(s) = name.strip_suffix(".jsonl.zst") {
                s
            } else if let Some(s) = name.strip_suffix(".jsonl") {
                s
            } else {
                continue;
            };

            if stem.ends_with(&format!(".{edge_id}")) {
                out.push(stem.to_string());
                if out.len() >= 2 {
                    break;
                }
            }
        }

        out.sort();
        out.dedup();
        out
    }

    /// Scans the remote filesystem for JSONL files matching the pattern `*.<edge_id>.jsonl(.zst)`.
    /// Returns a list of resolved file stems (without extensions) that end with `.<edge_id>`.
    ///
    /// This is used for incremental runs with subworkflows where edge files are namespaced
    /// as `<workflow_id>.<edge_id>.jsonl` instead of just `<edge_id>.jsonl`.
    async fn find_namespaced_jsonl_ids_async(&self, edge_id: &str) -> Vec<String> {
        let mut out = Vec::new();

        let list_result = self.storage.list(Some(self.root.as_path()), false).await;

        let mut stream = match list_result {
            Ok(stream) => stream,
            Err(e) => {
                tracing::debug!("Cannot list directory {} : {:?}", self.root.display(), e);
                return out;
            }
        };

        while let Some(result) = stream.next().await {
            if out.len() >= 2 {
                break;
            }

            let uri = match result {
                Ok(uri) => uri,
                Err(e) => {
                    tracing::debug!("Error reading list entry: {:?}", e);
                    continue;
                }
            };

            let path = uri.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            let stem = if let Some(s) = name.strip_suffix(".jsonl.zst") {
                s
            } else if let Some(s) = name.strip_suffix(".jsonl") {
                s
            } else {
                continue;
            };

            if stem.ends_with(&format!(".{edge_id}")) {
                out.push(stem.to_string());
            }
        }

        out.sort();
        out.dedup();
        out
    }

    fn looks_like_zstd(bytes: &[u8]) -> bool {
        // ZSTD frame magic number: 0xFD2FB528 (little endian in bytes: 28 B5 2F FD)
        bytes.len() >= 4
            && bytes[0] == 0x28
            && bytes[1] == 0xB5
            && bytes[2] == 0x2F
            && bytes[3] == 0xFD
    }

    fn decode_auto<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, [u8]>> {
        if Self::looks_like_zstd(bytes) {
            let v = zstd::stream::decode_all(bytes).map_err(Error::other)?;
            Ok(Cow::Owned(v))
        } else {
            Ok(Cow::Borrowed(bytes))
        }
    }

    pub fn read_jsonl_auto_sync<T>(&self, id: &str) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        // try both extensions, regardless of env
        let candidates = if self.use_compression {
            ["jsonl.zst", "jsonl"]
        } else {
            ["jsonl", "jsonl.zst"]
        };

        let mut last_err: Option<Error> = None;

        for ext in candidates {
            let p = self.id_to_location(id, ext);
            match self.storage.get_sync(p.as_path()) {
                Ok(bytes) => {
                    let data = self.decode_auto(bytes.as_ref())?;
                    let s = std::str::from_utf8(&data).map_err(Error::other)?;
                    let mut out = Vec::new();
                    for line in s.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        let obj: T = self.string_to_object(line)?;
                        out.push(obj);
                    }
                    return Ok(out);
                }
                Err(e) => {
                    last_err = Some(Error::other(e));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| Error::other("no candidate jsonl found")))
    }

    /// Rewrite all *.jsonl / *.jsonl.zst under this State's root directory in-place.
    /// It finds "filePath" keys recursively and replaces "/jobs/<prev>/" -> "/jobs/<cur>/".
    pub fn rewrite_feature_store_file_paths_in_root_dir(
        &self,
        previous_job_id: uuid::Uuid,
        job_id: uuid::Uuid,
    ) -> std::io::Result<()> {
        if !self.root.exists() {
            return Ok(());
        }

        let prev_jobs_seg = format!("/jobs/{}/", previous_job_id);
        let cur_jobs_seg = format!("/jobs/{}/", job_id);

        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            if !(file_name.ends_with(".jsonl") || file_name.ends_with(".jsonl.zst")) {
                continue;
            }

            self.rewrite_jsonl_file_in_place(&path, &prev_jobs_seg, &cur_jobs_seg)?;
        }

        Ok(())
    }

    fn rewrite_jsonl_file_in_place(
        &self,
        path: &std::path::Path,
        prev_jobs_seg: &str,
        cur_jobs_seg: &str,
    ) -> std::io::Result<()> {
        let tmp_path = tmp_sibling_path(path);

        // Ensure tmp cleanup on any error (RAII)
        struct TempGuard(PathBuf);
        impl Drop for TempGuard {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let _guard = TempGuard(tmp_path.clone());

        let raw = std::fs::read(path)?;
        let is_zst_by_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|n| n.ends_with(".zst"))
            .unwrap_or(false);
        let is_zst_by_magic = Self::looks_like_zstd(&raw);
        let should_write_zst = is_zst_by_name || is_zst_by_magic;

        let decoded = self.decode_auto(&raw)?;
        let text = std::str::from_utf8(&decoded).map_err(std::io::Error::other)?;

        let tmp_file = std::fs::File::create(&tmp_path)?;

        if should_write_zst {
            let writer = std::io::BufWriter::new(tmp_file);
            let mut enc =
                zstd::stream::Encoder::new(writer, ZSTD_LEVEL).map_err(std::io::Error::other)?;
            rewrite_jsonl_text(text, &mut enc, prev_jobs_seg, cur_jobs_seg)?;
            let mut w = enc.finish().map_err(std::io::Error::other)?;
            w.flush()?;
        } else {
            let mut w = std::io::BufWriter::new(tmp_file);
            rewrite_jsonl_text(text, &mut w, prev_jobs_seg, cur_jobs_seg)?;
            w.flush()?;
        }

        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

fn tmp_sibling_path(path: &Path) -> PathBuf {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return path.with_extension("tmp");
    };
    path.with_file_name(format!("{name}.tmp"))
}

fn rewrite_jsonl_text<W: std::io::Write>(
    text: &str,
    writer: &mut W,
    prev_jobs_seg: &str,
    cur_jobs_seg: &str,
) -> std::io::Result<()> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut v: serde_json::Value = serde_json::from_str(line).map_err(std::io::Error::other)?;
        rewrite_file_path_value(&mut v, prev_jobs_seg, cur_jobs_seg);
        serde_json::to_writer(&mut *writer, &v).map_err(std::io::Error::other)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn rewrite_file_path_value(v: &mut serde_json::Value, prev_jobs_seg: &str, cur_jobs_seg: &str) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map.iter_mut() {
                if k == "filePath" {
                    if let serde_json::Value::String(s) = val {
                        *s = rewrite_one_path(s, prev_jobs_seg, cur_jobs_seg);
                    }
                } else {
                    rewrite_file_path_value(val, prev_jobs_seg, cur_jobs_seg);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr.iter_mut() {
                rewrite_file_path_value(val, prev_jobs_seg, cur_jobs_seg);
            }
        }
        _ => {}
    }
}

fn rewrite_one_path(s: &str, prev_jobs_seg: &str, cur_jobs_seg: &str) -> String {
    // job id replacement
    let out = if s.contains(prev_jobs_seg) {
        s.replace(prev_jobs_seg, cur_jobs_seg)
    } else {
        s.to_string()
    };

    // local existence checks (file:// supported)
    let (plain, prefix) = if let Some(rest) = out.strip_prefix("file://") {
        (rest.to_string(), "file://")
    } else {
        (out.clone(), "")
    };

    if std::path::Path::new(&plain).exists() {
        return out;
    }

    // artifacts <-> temp-artifacts fallback
    if plain.contains("/artifacts/") {
        let alt = plain.replace("/artifacts/", "/temp-artifacts/");
        if std::path::Path::new(&alt).exists() {
            return format!("{prefix}{alt}");
        }
    }
    if plain.contains("/temp-artifacts/") {
        let alt = plain.replace("/temp-artifacts/", "/artifacts/");
        if std::path::Path::new(&alt).exists() {
            return format!("{prefix}{alt}");
        }
    }

    // This is mainly to handle temp-id subdirs (temp-artifacts/<temp-id>/...).
    if let Some(resolved) = resolve_in_job_temp_artifacts_by_basename(&plain) {
        return format!("{prefix}{resolved}");
    }

    out
}

/// Try to locate `original` under the same job's `temp-artifacts/**` by basename.
fn resolve_in_job_temp_artifacts_by_basename(original: &str) -> Option<String> {
    let original_path = Path::new(original);
    let basename = original_path.file_name()?.to_string_lossy().to_string();

    let job_root = job_root_from_any_path(original_path)?;
    let temp_artifacts_root = job_root.join("temp-artifacts");
    if !temp_artifacts_root.exists() {
        return None;
    }

    let mut hits = Vec::new();
    collect_files_named(&temp_artifacts_root, &basename, &mut hits);

    match hits.len() {
        1 => Some(hits[0].to_string_lossy().to_string()),
        n if n > 1 => {
            tracing::warn!(
                "Multiple files with the same basename found under temp-artifacts; cannot resolve uniquely. hits={}, target={}, root={}, original={}",
                n,
                basename,
                temp_artifacts_root.display(),
                original
            );
            None
        }
        _ => None,
    }
}

/// Extract `.../jobs/<job_id>` from any path containing that segment.
fn job_root_from_any_path(p: &std::path::Path) -> Option<PathBuf> {
    let comps: Vec<String> = p
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    let jobs_idx = comps.iter().position(|s| s == "jobs")?;
    let job_id_idx = jobs_idx + 1;
    if job_id_idx >= comps.len() {
        return None;
    }

    let mut out = PathBuf::new();
    for (i, c) in p.components().enumerate() {
        out.push(c.as_os_str());
        if i == job_id_idx {
            break;
        }
    }
    Some(out)
}

/// Recursively collect files whose basename matches `target_name`.
fn collect_files_named(dir: &Path, target_name: &str, out: &mut Vec<PathBuf>) {
    fn inner(dir: &std::path::Path, target_name: &str, out: &mut Vec<PathBuf>, depth: usize) {
        if depth > MAX_DIRECTORY_DEPTH || out.len() >= MAX_HITS {
            return;
        }

        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in rd.flatten() {
            if out.len() >= MAX_HITS {
                return;
            }

            let path = entry.path();
            let Ok(ft) = entry.file_type() else { continue };

            if ft.is_dir() {
                inner(&path, target_name, out, depth + 1);
                continue;
            }

            if ft.is_file()
                && path
                    .file_name()
                    .map(|n| n.to_string_lossy() == target_name)
                    .unwrap_or(false)
            {
                out.push(path);
            }
        }
    }

    inner(dir, target_name, out, 0);
}

#[cfg(test)]
impl State {
    pub(crate) fn new_for_test(
        root: &Uri,
        storage_resolver: &StorageResolver,
        use_compression: bool,
    ) -> std::io::Result<Self> {
        let mut s = State::new(root, storage_resolver)?;
        s.use_compression = use_compression;
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::Builder;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        x: i32,
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let storage_resolver = Arc::new(StorageResolver::new());

        let state = State::new(&Uri::for_test("ram:///workflows"), &storage_resolver).unwrap();
        let data = Data { x: 42 };
        state.save(&data, "test").await.unwrap();
        let result: Data = state.get("test").await.unwrap();
        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn test_state_write_read_zstd_enabled() {
        let storage_resolver = Arc::new(StorageResolver::new());

        let state =
            State::new_for_test(&Uri::for_test("ram:///workflows"), &storage_resolver, true)
                .unwrap();
        let data = Data { x: 42 };
        state.save(&data, "test").await.unwrap();
        let result: Data = state.get("test").await.unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_job_root_from_any_path() {
        let p = Path::new("/a/b/jobs/1234/temp-artifacts/x/y.csv");
        let root = job_root_from_any_path(p).unwrap();
        assert_eq!(root, PathBuf::from("/a/b/jobs/1234"));

        let p2 = Path::new("/a/b/nope/1234/temp-artifacts/x/y.csv");
        assert!(job_root_from_any_path(p2).is_none());
    }

    #[test]
    fn test_resolve_in_job_temp_artifacts_by_basename_unique() {
        let temp_dir = Builder::new()
            .prefix("test_resolve_unique_")
            .tempdir()
            .unwrap();

        let job_root = temp_dir.path().join("jobs").join("jobid");
        let target = job_root
            .join("temp-artifacts")
            .join("tid")
            .join("year-2024.csv");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "ok").unwrap();

        let original = job_root.join("artifacts").join("year-2024.csv");
        std::fs::create_dir_all(original.parent().unwrap()).unwrap();

        let resolved =
            resolve_in_job_temp_artifacts_by_basename(original.to_str().unwrap()).unwrap();
        assert_eq!(PathBuf::from(resolved), target);
    }

    #[test]
    fn test_resolve_in_job_temp_artifacts_by_basename_multiple() {
        let temp_dir = Builder::new()
            .prefix("test_resolve_multiple_")
            .tempdir()
            .unwrap();

        let job_root = temp_dir.path().join("jobs").join("jobid");
        let a = job_root
            .join("temp-artifacts")
            .join("t1")
            .join("year-2024.csv");
        let b = job_root
            .join("temp-artifacts")
            .join("t2")
            .join("year-2024.csv");

        std::fs::create_dir_all(a.parent().unwrap()).unwrap();
        std::fs::create_dir_all(b.parent().unwrap()).unwrap();
        std::fs::write(&a, "a").unwrap();
        std::fs::write(&b, "b").unwrap();

        let original = job_root.join("artifacts").join("year-2024.csv");
        std::fs::create_dir_all(original.parent().unwrap()).unwrap();

        assert!(resolve_in_job_temp_artifacts_by_basename(original.to_str().unwrap()).is_none());
    }
}
