//! Point primitives: [`Builder::push_points`] and the raw buffer, view and
//! accessor calls a caller lays their vertex streams out with.

use gltf::json;
use gltf::json::validation::{Checked, USize64};

use super::{Builder, Extension, MaterialDesc, PendingPrimitive, PrimitiveHandle, pad_to};

pub(super) struct PointsPrimitive {
    attributes: Vec<(json::mesh::Semantic, json::Index<json::Accessor>)>,
    material: json::Index<json::Material>,
    pub(super) extensions: Vec<Extension>,
}

/// Index the data-less fallback buffer takes, after the BIN chunk's buffer 0.
const FALLBACK_BUFFER: u32 = 1;

impl Builder {
    /// Append `bytes` to the BIN chunk without a view, `align`ed, and return
    /// their byte offset.
    pub fn push_bytes(&mut self, bytes: &[u8], align: usize) -> usize {
        pad_to(&mut self.bin, align);
        let byte_offset = self.bin.len();
        self.bin.extend_from_slice(bytes);
        byte_offset
    }

    /// Append `bytes` as an interleaved vertex view of `byte_stride`.
    pub fn push_vertex_view(
        &mut self,
        bytes: &[u8],
        byte_stride: usize,
    ) -> json::Index<json::buffer::View> {
        let byte_offset = self.push_bytes(bytes, 4);
        self.push_view(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64::from(bytes.len()),
            byte_offset: Some(USize64::from(byte_offset)),
            byte_stride: Some(json::buffer::Stride(byte_stride)),
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        })
    }

    /// A vertex view of `byte_length` bytes on a buffer that holds no data,
    /// whose bytes `extension` (with `value` on the view) supplies instead. The
    /// buffer is marked `{extension: {"fallback": true}}`, so `extension` must
    /// also be required.
    pub fn push_fallback_view(
        &mut self,
        byte_length: usize,
        byte_stride: usize,
        extension: &'static str,
        value: serde_json::Value,
    ) -> json::Index<json::buffer::View> {
        let reserved = self.fallback.map_or(0, |(len, _)| len).next_multiple_of(4);
        self.fallback = Some((reserved + byte_length, extension));
        let mut extensions = json::extensions::buffer::View::default();
        extensions.others.insert(extension.to_string(), value);
        self.mark_extension_used(extension);
        self.push_view(json::buffer::View {
            buffer: json::Index::new(FALLBACK_BUFFER),
            byte_length: USize64::from(byte_length),
            byte_offset: Some(USize64::from(reserved)),
            byte_stride: Some(json::buffer::Stride(byte_stride)),
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ArrayBuffer)),
            extensions: Some(extensions),
            extras: Default::default(),
        })
    }

    fn push_view(&mut self, view: json::buffer::View) -> json::Index<json::buffer::View> {
        self.root.push(view)
    }

    pub(super) fn push_fallback_buffer(&mut self, byte_length: usize, extension: &'static str) {
        let mut extensions = json::extensions::buffer::Buffer::default();
        extensions.others.insert(
            extension.to_string(),
            serde_json::json!({ "fallback": true }),
        );
        let index = self.root.push(json::Buffer {
            byte_length: USize64::from(byte_length),
            name: None,
            uri: None,
            extensions: Some(extensions),
            extras: Default::default(),
        });
        debug_assert_eq!(index.value(), FALLBACK_BUFFER as usize);
    }

    /// Append an accessor as given.
    pub fn push_accessor(&mut self, accessor: json::Accessor) -> json::Index<json::Accessor> {
        self.root.push(accessor)
    }

    /// Add a `POINTS` primitive over accessors already pushed.
    pub fn push_points(
        &mut self,
        attributes: Vec<(json::mesh::Semantic, json::Index<json::Accessor>)>,
        material: MaterialDesc,
    ) -> PrimitiveHandle {
        let material = self.push_material(material);
        self.primitives
            .push(PendingPrimitive::Points(PointsPrimitive {
                attributes,
                material,
                extensions: Vec::new(),
            }));
        PrimitiveHandle(self.primitives.len() - 1)
    }

    pub(super) fn finish_points(&mut self, p: PointsPrimitive) -> json::mesh::Primitive {
        let attributes = p
            .attributes
            .into_iter()
            .map(|(semantic, accessor)| (Checked::Valid(semantic), accessor))
            .collect();
        let mut extensions = json::extensions::mesh::Primitive::default();
        for ext in p.extensions {
            extensions.others.insert(ext.name.to_string(), ext.value);
            self.mark_extension_used(ext.name);
        }
        json::mesh::Primitive {
            attributes,
            extensions: (!extensions.others.is_empty()).then_some(extensions),
            extras: Default::default(),
            indices: None,
            material: Some(p.material),
            mode: Checked::Valid(json::mesh::Mode::Points),
            targets: None,
        }
    }
}
