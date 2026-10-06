//! Disk-backed store of the writer's per-feature render input.
//!
//! Each feature is encoded once, at `process()`, into an append-only file in
//! the executor cache; only its [`GeoBox`] and its byte range stay in memory.
//! Each reader opens its own handle on that file, so any number of threads can
//! decode records concurrently from a shared `&FeatureStore`.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use reearth_flow_gltf::tiles::metadata::{self, MetadataOptions};
use reearth_flow_types::datetime::DateTime;
use reearth_flow_types::geometry::GeometryValue;
use reearth_flow_types::{AttributeValue, Feature};

use super::appearance::{ResolvedMaterial, TextureSource};
use super::mesh::{self, ExtractOptions, ExtractedMesh};
use super::quadtree::GeoBox;

/// Bytes buffered in memory before they are appended to the temp file.
const FLUSH_THRESHOLD: usize = 8 * 1024 * 1024;

/// One feature's render input as the cell builder consumes it.
pub(super) struct StoredFeature {
    pub(super) feature_type: Option<String>,
    /// Flattened leaf attributes, keyed by attribute path.
    pub(super) attributes: BTreeMap<String, AttributeValue>,
    pub(super) mesh: ExtractedMesh,
}

impl StoredFeature {
    /// `feature`'s render input with its placement extent, or `None` when it
    /// has no renderable CityGML geometry.
    pub(super) fn ingest(
        feature: &Feature,
        metadata: MetadataOptions,
        extract: ExtractOptions,
    ) -> Option<(GeoBox, Self)> {
        let GeometryValue::CityGmlGeometry(city_gml) = &feature.geometry.value else {
            return None;
        };
        let (mesh, bounds) = mesh::extract(city_gml, extract)?;
        let stored = StoredFeature {
            feature_type: feature.feature_type(),
            attributes: metadata::flatten_attributes(feature, metadata),
            mesh,
        };
        Some((bounds, stored))
    }
}

struct Entry {
    offset: u64,
    len: usize,
    bounds: GeoBox,
}

/// Append-only feature records; see the module docs.
pub(super) struct FeatureStore {
    /// Where the records file is created on first flush; removed on drop.
    path: PathBuf,
    /// Write handle, opened on first flush.
    file: Option<File>,
    /// Bytes already written to `file`.
    flushed: u64,
    /// Encoded records not yet written to `file`, starting at `flushed`.
    tail: Vec<u8>,
    entries: Vec<Entry>,
}

impl std::fmt::Debug for FeatureStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureStore")
            .field("path", &self.path)
            .field("len", &self.entries.len())
            .field("bytes", &(self.flushed + self.tail.len() as u64))
            .finish_non_exhaustive()
    }
}

impl Drop for FeatureStore {
    fn drop(&mut self) {
        if self.file.take().is_some() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl FeatureStore {
    /// An empty store whose records file will be a new uniquely named file in
    /// `dir`; nothing touches the disk until the first flush.
    pub(super) fn new(dir: &Path) -> Self {
        Self {
            path: dir.join(format!("features-{}.bin", uuid::Uuid::new_v4())),
            file: None,
            flushed: 0,
            tail: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Geographic extent of record `i`.
    pub(super) fn bounds(&self, i: usize) -> &GeoBox {
        &self.entries[i].bounds
    }

    /// Total encoded bytes held, on disk and buffered.
    pub(super) fn bytes(&self) -> u64 {
        self.flushed + self.tail.len() as u64
    }

    /// Append `feature`, placed by `bounds`.
    pub(super) fn push(&mut self, bounds: GeoBox, feature: &StoredFeature) -> io::Result<()> {
        let start = self.tail.len();
        encode_feature(&mut self.tail, feature);
        self.entries.push(Entry {
            offset: self.flushed + start as u64,
            len: self.tail.len() - start,
            bounds,
        });
        if self.tail.len() >= FLUSH_THRESHOLD {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.tail.is_empty() {
            return Ok(());
        }
        let file = match &mut self.file {
            Some(file) => file,
            None => {
                if let Some(dir) = self.path.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                self.file.insert(File::create(&self.path)?)
            }
        };
        file.write_all(&self.tail)?;
        self.flushed += self.tail.len() as u64;
        self.tail.clear();
        Ok(())
    }

    /// A reader over the records pushed so far. Each reader holds its own file
    /// handle, so readers on different threads never contend.
    pub(super) fn reader(&self) -> io::Result<StoreReader<'_>> {
        let file = match self.file {
            Some(_) => Some(File::open(&self.path)?),
            None => None,
        };
        Ok(StoreReader { store: self, file })
    }
}

/// One thread's read handle on a [`FeatureStore`].
pub(super) struct StoreReader<'a> {
    store: &'a FeatureStore,
    /// `None` while every record is still in the store's in-memory tail.
    file: Option<File>,
}

impl StoreReader<'_> {
    /// Decode record `i`.
    pub(super) fn read(&mut self, i: usize) -> io::Result<StoredFeature> {
        let store = self.store;
        let entry = &store.entries[i];
        if entry.offset >= store.flushed {
            let start = (entry.offset - store.flushed) as usize;
            return decode_feature(&store.tail[start..start + entry.len]);
        }
        let file = self
            .file
            .as_mut()
            .expect("a record below `flushed` was written to the file");
        let mut buf = vec![0u8; entry.len];
        file.seek(SeekFrom::Start(entry.offset))?;
        file.read_exact(&mut buf)?;
        decode_feature(&buf)
    }
}

// Records are read back only by this process, so the encoding is native-endian
// and carries no versioning.

fn put_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_ne_bytes());
}

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    put_u32(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    put_bytes(out, s.as_bytes());
}

fn put_pods<T: bytemuck::Pod>(out: &mut Vec<u8>, values: &[T]) {
    put_u32(out, values.len() as u32);
    out.extend_from_slice(bytemuck::cast_slice(values));
}

fn encode_feature(out: &mut Vec<u8>, feature: &StoredFeature) {
    match &feature.feature_type {
        Some(ty) => {
            put_u8(out, 1);
            put_str(out, ty);
        }
        None => put_u8(out, 0),
    }
    put_u32(out, feature.attributes.len() as u32);
    for (key, value) in &feature.attributes {
        put_str(out, key);
        encode_value(out, value);
    }

    let mesh = &feature.mesh;
    encode_positions(out, &mesh.positions);
    put_pods(out, &mesh.indices);
    put_pods(out, &mesh.polygon_tris);
    put_pods(out, &mesh.polygon_normals);
    put_pods(out, &mesh.polygon_material);
    put_pods(out, &mesh.uvs);
    put_u32(out, mesh.materials.len() as u32);
    for material in &mesh.materials {
        put_pods(out, &material.base_color_factor);
        put_pods(out, &[material.metallic_factor, material.roughness_factor]);
        match &material.base_texture {
            Some(TextureSource::File(path)) => {
                put_u8(out, 1);
                put_bytes(out, path.as_os_str().as_encoded_bytes());
            }
            None => put_u8(out, 0),
        }
    }
}

/// Largest quantized coordinate; a position's offset into its feature's ECEF
/// box is stored as a fraction of the box extent in `0..=QUANT_MAX`.
const QUANT_MAX: f64 = u32::MAX as f64;

/// Writes `positions` as the f64 ECEF box around them followed by each
/// vertex's u32-quantized offset into that box. The round-trip error per axis
/// is at most `extent / (2 * u32::MAX)`, about 1.2 µm for a 10 km feature.
fn encode_positions(out: &mut Vec<u8>, positions: &[[f64; 3]]) {
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for p in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    if positions.is_empty() {
        (min, max) = ([0.0; 3], [0.0; 3]);
    }
    put_pods(out, &min);
    put_pods(out, &max);
    let scale = std::array::from_fn::<f64, 3, _>(|axis| {
        let extent = max[axis] - min[axis];
        if extent > 0.0 {
            QUANT_MAX / extent
        } else {
            0.0
        }
    });
    let quantized: Vec<[u32; 3]> = positions
        .iter()
        .map(|p| {
            std::array::from_fn(|axis| {
                ((p[axis] - min[axis]) * scale[axis])
                    .round()
                    .clamp(0.0, QUANT_MAX) as u32
            })
        })
        .collect();
    put_pods(out, &quantized);
}

fn decode_positions(r: &mut Reader<'_>) -> io::Result<Vec<[f64; 3]>> {
    let min: [f64; 3] = r.array()?;
    let max: [f64; 3] = r.array()?;
    let step = std::array::from_fn::<f64, 3, _>(|axis| (max[axis] - min[axis]) / QUANT_MAX);
    let quantized: Vec<[u32; 3]> = r.pods()?;
    Ok(quantized
        .iter()
        .map(|q| std::array::from_fn(|axis| min[axis] + q[axis] as f64 * step[axis]))
        .collect())
}

const TAG_NULL: u8 = 0;
const TAG_BOOL: u8 = 1;
const TAG_NUMBER: u8 = 2;
const TAG_STRING: u8 = 3;
const TAG_DATETIME: u8 = 4;
const TAG_ARRAY: u8 = 5;
const TAG_MAP: u8 = 6;
const TAG_BYTES: u8 = 7;

fn encode_value(out: &mut Vec<u8>, value: &AttributeValue) {
    match value {
        AttributeValue::Null => put_u8(out, TAG_NULL),
        AttributeValue::Bool(b) => {
            put_u8(out, TAG_BOOL);
            put_u8(out, *b as u8);
        }
        AttributeValue::Number(n) => {
            put_u8(out, TAG_NUMBER);
            put_str(out, &n.to_string());
        }
        AttributeValue::String(s) => {
            put_u8(out, TAG_STRING);
            put_str(out, s);
        }
        AttributeValue::DateTime(dt) => {
            put_u8(out, TAG_DATETIME);
            put_str(out, &dt.to_raw());
        }
        AttributeValue::Array(values) => {
            put_u8(out, TAG_ARRAY);
            put_u32(out, values.len() as u32);
            for v in values {
                encode_value(out, v);
            }
        }
        AttributeValue::Map(map) => {
            put_u8(out, TAG_MAP);
            put_u32(out, map.len() as u32);
            for (k, v) in map {
                put_str(out, k);
                encode_value(out, v);
            }
        }
        AttributeValue::Bytes(bytes) => {
            put_u8(out, TAG_BYTES);
            put_bytes(out, bytes);
        }
    }
}

fn invalid(what: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("corrupt feature record: {what}"),
    )
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.0.len() < n {
            return Err(invalid("truncated"));
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }

    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> io::Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_ne_bytes(bytes.try_into().expect("took 4 bytes")))
    }

    fn bytes(&mut self) -> io::Result<&'a [u8]> {
        let len = self.u32()? as usize;
        self.take(len)
    }

    fn string(&mut self) -> io::Result<String> {
        let bytes = self.bytes()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| invalid("non-UTF-8 string"))
    }

    fn pods<T: bytemuck::Pod>(&mut self) -> io::Result<Vec<T>> {
        let len = self.u32()? as usize;
        let bytes = self.take(len * std::mem::size_of::<T>())?;
        Ok(bytemuck::pod_collect_to_vec(bytes))
    }

    fn array<T: bytemuck::Pod, const N: usize>(&mut self) -> io::Result<[T; N]> {
        let values: Vec<T> = self.pods()?;
        values.try_into().map_err(|_| invalid("array length"))
    }
}

fn decode_feature(bytes: &[u8]) -> io::Result<StoredFeature> {
    let mut r = Reader(bytes);
    let feature_type = match r.u8()? {
        0 => None,
        _ => Some(r.string()?),
    };
    let attribute_count = r.u32()? as usize;
    let mut attributes = BTreeMap::new();
    for _ in 0..attribute_count {
        let key = r.string()?;
        attributes.insert(key, decode_value(&mut r)?);
    }

    let positions = decode_positions(&mut r)?;
    let indices = r.pods()?;
    let polygon_tris = r.pods()?;
    let polygon_normals = r.pods()?;
    let polygon_material = r.pods()?;
    let uvs = r.pods()?;
    let material_count = r.u32()? as usize;
    let mut materials = Vec::with_capacity(material_count);
    for _ in 0..material_count {
        let base_color_factor = r.array()?;
        let [metallic_factor, roughness_factor] = r.array()?;
        let base_texture = match r.u8()? {
            0 => None,
            _ => {
                let bytes = r.bytes()?;
                // SAFETY: `bytes` came from `as_encoded_bytes` in this process
                // (see `encode_feature`), on the same platform and Rust version.
                let os = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(bytes) };
                Some(TextureSource::File(PathBuf::from(os)))
            }
        };
        materials.push(ResolvedMaterial {
            base_color_factor,
            metallic_factor,
            roughness_factor,
            base_texture,
        });
    }
    if !r.0.is_empty() {
        return Err(invalid("trailing bytes"));
    }

    Ok(StoredFeature {
        feature_type,
        attributes,
        mesh: ExtractedMesh {
            positions,
            indices,
            polygon_tris,
            polygon_normals,
            polygon_material,
            materials,
            uvs,
        },
    })
}

fn decode_value(r: &mut Reader<'_>) -> io::Result<AttributeValue> {
    Ok(match r.u8()? {
        TAG_NULL => AttributeValue::Null,
        TAG_BOOL => AttributeValue::Bool(r.u8()? != 0),
        TAG_NUMBER => AttributeValue::Number(
            serde_json::Number::from_str(&r.string()?).map_err(|_| invalid("number"))?,
        ),
        TAG_STRING => AttributeValue::String(r.string()?),
        TAG_DATETIME => AttributeValue::DateTime(
            DateTime::from_str(&r.string()?).map_err(|_| invalid("datetime"))?,
        ),
        TAG_ARRAY => {
            let len = r.u32()? as usize;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(decode_value(r)?);
            }
            AttributeValue::Array(values)
        }
        TAG_MAP => {
            let len = r.u32()? as usize;
            let mut map = std::collections::HashMap::with_capacity(len);
            for _ in 0..len {
                let key = r.string()?;
                map.insert(key, decode_value(r)?);
            }
            AttributeValue::Map(map)
        }
        TAG_BYTES => AttributeValue::Bytes(bytes::Bytes::copy_from_slice(r.bytes()?)),
        _ => return Err(invalid("attribute tag")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;

    fn bounds(i: usize) -> GeoBox {
        let v = i as f64;
        GeoBox {
            west: v,
            south: v,
            east: v + 1.0,
            north: v + 1.0,
            min_height: 0.0,
            max_height: 10.0,
        }
    }

    fn feature(i: usize) -> StoredFeature {
        let attributes = BTreeMap::from([
            ("null".to_string(), AttributeValue::Null),
            (
                "bool".to_string(),
                AttributeValue::Bool(i.is_multiple_of(2)),
            ),
            (
                "int".to_string(),
                AttributeValue::Number(serde_json::Number::from(i as i64 - 7)),
            ),
            (
                "float".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(i as f64 + 0.25).unwrap()),
            ),
            (
                "string".to_string(),
                AttributeValue::String(format!("f{i}")),
            ),
            (
                "date".to_string(),
                AttributeValue::DateTime(DateTime::from_str("2024-03-01").unwrap()),
            ),
            (
                "bytes".to_string(),
                AttributeValue::Bytes(bytes::Bytes::from(vec![i as u8, 1, 2])),
            ),
            (
                "array".to_string(),
                AttributeValue::Array(vec![AttributeValue::Map(
                    [("k".to_string(), AttributeValue::String("v".to_string()))].into(),
                )]),
            ),
        ]);
        StoredFeature {
            feature_type: (!i.is_multiple_of(3)).then(|| format!("type{}", i % 3)),
            attributes,
            mesh: ExtractedMesh {
                positions: vec![
                    [-3_955_000.0, 3_350_000.0, 3_700_000.0 + i as f64],
                    [-3_955_001.0, 3_350_000.0, 3_700_000.0],
                    [-3_955_000.0, 3_350_000.5, 3_700_000.0],
                ],
                indices: vec![[0, 1, 2]],
                polygon_tris: vec![1],
                polygon_normals: vec![[0.0, 0.0, 1.0]],
                polygon_material: vec![0],
                materials: vec![ResolvedMaterial {
                    base_color_factor: [1.0, 0.5, 0.25, 1.0],
                    metallic_factor: 0.0,
                    roughness_factor: 0.9,
                    base_texture: (!i.is_multiple_of(2))
                        .then(|| TextureSource::File(PathBuf::from(format!("/tex/{i}.jpg")))),
                }],
                uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            },
        }
    }

    fn assert_positions_close(a: &[[f64; 3]], b: &[[f64; 3]], tolerance: f64) {
        assert_eq!(a.len(), b.len());
        for (a, b) in a.iter().zip(b) {
            for axis in 0..3 {
                assert!(
                    (a[axis] - b[axis]).abs() <= tolerance,
                    "{a:?} vs {b:?} differ by more than {tolerance}"
                );
            }
        }
    }

    fn assert_same(a: &StoredFeature, b: &StoredFeature) {
        assert_eq!(a.feature_type, b.feature_type);
        assert_eq!(
            serde_json::to_value(&a.attributes).unwrap(),
            serde_json::to_value(&b.attributes).unwrap()
        );
        let (a, b) = (&a.mesh, &b.mesh);
        assert_positions_close(&a.positions, &b.positions, 1e-6);
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.polygon_tris, b.polygon_tris);
        assert_eq!(a.polygon_normals, b.polygon_normals);
        assert_eq!(a.polygon_material, b.polygon_material);
        assert_eq!(a.uvs, b.uvs);
        assert_eq!(a.materials.len(), b.materials.len());
        for (a, b) in a.materials.iter().zip(&b.materials) {
            assert_eq!(a.base_color_factor, b.base_color_factor);
            assert_eq!(a.metallic_factor, b.metallic_factor);
            assert_eq!(a.roughness_factor, b.roughness_factor);
            let path = |m: &ResolvedMaterial| {
                m.base_texture
                    .as_ref()
                    .map(|TextureSource::File(p)| p.clone())
            };
            assert_eq!(path(a), path(b));
        }
    }

    // Records round-trip whether they were flushed to the file or still sit
    // in the in-memory tail.
    #[test]
    fn records_round_trip_from_file_and_tail() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FeatureStore::new(dir.path());
        for i in 0..5 {
            store.push(bounds(i), &feature(i)).unwrap();
        }
        store.flush().unwrap();
        for i in 5..8 {
            store.push(bounds(i), &feature(i)).unwrap();
        }
        assert!(store.file.is_some() && !store.tail.is_empty());

        assert_eq!(store.len(), 8);
        let mut reader = store.reader().unwrap();
        for i in 0..8 {
            assert_eq!(store.bounds(i).west, i as f64);
            assert_same(&reader.read(i).unwrap(), &feature(i));
        }
    }

    // Decoded attribute values keep their variant, including the ones the
    // untagged serde form would collapse (a datetime would read back as a
    // string).
    #[test]
    fn attribute_variants_survive() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FeatureStore::new(dir.path());
        store.push(bounds(0), &feature(4)).unwrap();
        let read = store.reader().unwrap().read(0).unwrap();
        assert!(matches!(
            read.attributes["date"],
            AttributeValue::DateTime(_)
        ));
        assert!(matches!(read.attributes["bytes"], AttributeValue::Bytes(_)));
        assert!(
            matches!(read.attributes["int"], AttributeValue::Number(ref n) if n.as_i64() == Some(-3))
        );
    }

    // A feature spanning 10 km in every axis round-trips to within the
    // quantization bound, well below the f32 tile positions it ends up in.
    #[test]
    fn large_feature_positions_keep_micrometre_precision() {
        let base = [-3_955_123.456_789, 3_350_987.654_321, 3_700_555.555_555];
        let positions: Vec<[f64; 3]> = (0..=100)
            .map(|k| {
                let t = k as f64 * 100.0 + 0.123_456_789;
                [base[0] + t, base[1] - t * 0.7, base[2] + t * 0.3]
            })
            .chain([
                [base[0], base[1], base[2]],
                [base[0] + 10_000.0, base[1] - 10_000.0, base[2] + 10_000.0],
            ])
            .collect();
        let mut stored = feature(0);
        stored.mesh.positions = positions.clone();
        let dir = tempfile::tempdir().unwrap();
        let mut store = FeatureStore::new(dir.path());
        store.push(bounds(0), &stored).unwrap();
        store.flush().unwrap();

        let read = store.reader().unwrap().read(0).unwrap();
        assert_positions_close(&read.mesh.positions, &positions, 10_000.0 / u32::MAX as f64);
    }

    // An axis with zero extent (a flat feature, or a single point) decodes
    // exactly instead of dividing by zero.
    #[test]
    fn zero_extent_axis_round_trips_exactly() {
        let mut stored = feature(0);
        stored.mesh.positions = vec![[1.5, 2.5, 3.5], [1.5, 7.25, 3.5]];
        let dir = tempfile::tempdir().unwrap();
        let mut store = FeatureStore::new(dir.path());
        store.push(bounds(0), &stored).unwrap();

        let read = store.reader().unwrap().read(0).unwrap();
        for p in &read.mesh.positions {
            assert_eq!((p[0], p[2]), (1.5, 3.5));
        }
        assert_positions_close(&read.mesh.positions, &stored.mesh.positions, 1e-9);
    }

    // The records file lives in the given directory and is removed with the
    // store.
    #[test]
    fn records_file_is_created_in_dir_and_removed_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FeatureStore::new(dir.path());
        store.push(bounds(0), &feature(0)).unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        store.flush().unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        drop(store);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn concurrent_reads_see_every_record() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FeatureStore::new(dir.path());
        for i in 0..2000 {
            store.push(bounds(i), &feature(i)).unwrap();
            if i % 300 == 0 {
                store.flush().unwrap();
            }
        }
        // Every index is read twice, so threads also race on the same record.
        (0..4000usize).into_par_iter().for_each_init(
            || store.reader().unwrap(),
            |reader, k| {
                let i = k % 2000;
                assert_same(&reader.read(i).unwrap(), &feature(i));
            },
        );
    }
}
