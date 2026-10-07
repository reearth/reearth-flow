use std::collections::{HashMap, HashSet};
use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use reearth_flow_citygml::schema::{ClassModel, QName, SchemaSet};
use reearth_flow_types::material::X3DMaterial;

// The seam is geometry-neutral and shared; only the `posList` formatter is
// world-specific, and `converter` resolves to the compiled world's module, so
// this file needs no `cfg` to pick the right one.
use super::converter::format_pos_list;
use super::model::{
    AppearanceBundle, BoundingEnvelope, CityObjectType, GeometryEntry, GmlElement, GmlSolid,
    GmlSurface, GmlTexture,
};
use crate::errors::SinkError;

/// Written when the source recorded no theme name; the theme PLATEAU's textured
/// models use.
const FALLBACK_THEME: &str = "rgbTexture";

/// Collected per-surface appearance info, built while writing geometry.
struct SurfaceAppearance {
    surface_id: String,
    material_idx: Option<u32>,
    texture_idx: Option<u32>,
    uv_exterior: Vec<[f64; 2]>,
    uv_interiors: Vec<Vec<[f64; 2]>>,
}

const CITYGML_2_NAMESPACES: &[(&str, &str)] = &[
    ("xmlns:core", "http://www.opengis.net/citygml/2.0"),
    ("xmlns:gml", "http://www.opengis.net/gml"),
    ("xmlns:bldg", "http://www.opengis.net/citygml/building/2.0"),
    (
        "xmlns:tran",
        "http://www.opengis.net/citygml/transportation/2.0",
    ),
    ("xmlns:brid", "http://www.opengis.net/citygml/bridge/2.0"),
    ("xmlns:tun", "http://www.opengis.net/citygml/tunnel/2.0"),
    ("xmlns:wtr", "http://www.opengis.net/citygml/waterbody/2.0"),
    ("xmlns:luse", "http://www.opengis.net/citygml/landuse/2.0"),
    ("xmlns:veg", "http://www.opengis.net/citygml/vegetation/2.0"),
    (
        "xmlns:frn",
        "http://www.opengis.net/citygml/cityfurniture/2.0",
    ),
    ("xmlns:dem", "http://www.opengis.net/citygml/relief/2.0"),
    ("xmlns:gen", "http://www.opengis.net/citygml/generics/2.0"),
    ("xmlns:app", "http://www.opengis.net/citygml/appearance/2.0"),
    ("xmlns:xlink", "http://www.w3.org/1999/xlink"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
];

/// Namespaces a written property may use beyond those declared on the root.
/// Declared on the outermost element that uses them instead, so documents
/// without such content keep exactly the root they always had.
const LOCALLY_DECLARED_NAMESPACES: &[(&str, &str)] =
    &[("xmlns:xAL", "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0")];

/// Paired namespace and schema URLs, in the `http://` spelling PLATEAU's own
/// documents use. Lets the `XML Validator` action resolve our output.
const CITYGML_2_SCHEMA_LOCATION: &str = concat!(
    "http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd ",
    "http://www.opengis.net/citygml/2.0 ",
    "http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd ",
    "http://www.opengis.net/citygml/building/2.0 ",
    "http://schemas.opengis.net/citygml/building/2.0/building.xsd ",
    "http://www.opengis.net/citygml/transportation/2.0 ",
    "http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd ",
    "http://www.opengis.net/citygml/bridge/2.0 ",
    "http://schemas.opengis.net/citygml/bridge/2.0/bridge.xsd ",
    "http://www.opengis.net/citygml/tunnel/2.0 ",
    "http://schemas.opengis.net/citygml/tunnel/2.0/tunnel.xsd ",
    "http://www.opengis.net/citygml/waterbody/2.0 ",
    "http://schemas.opengis.net/citygml/waterbody/2.0/waterBody.xsd ",
    "http://www.opengis.net/citygml/landuse/2.0 ",
    "http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd ",
    "http://www.opengis.net/citygml/vegetation/2.0 ",
    "http://schemas.opengis.net/citygml/vegetation/2.0/vegetation.xsd ",
    "http://www.opengis.net/citygml/cityfurniture/2.0 ",
    "http://schemas.opengis.net/citygml/cityfurniture/2.0/cityFurniture.xsd ",
    "http://www.opengis.net/citygml/relief/2.0 ",
    "http://schemas.opengis.net/citygml/relief/2.0/relief.xsd ",
    "http://www.opengis.net/citygml/generics/2.0 ",
    "http://schemas.opengis.net/citygml/generics/2.0/generics.xsd ",
    "http://www.opengis.net/citygml/appearance/2.0 ",
    "http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd",
);

pub struct CityGmlXmlWriter<W: Write> {
    writer: Writer<W>,
    srs_name: String,
    /// One running count per prefix, so numbering a city object never shifts
    /// the `poly_N` ids beneath it.
    id_counters: HashMap<String, u64>,
    /// `xs:ID` is unique per document, so every id the writer emits is claimed here.
    used_ids: HashSet<String>,
    /// Surfaces dropped by the LOD1 exclusive-shell rule, reported by the sink so
    /// the drop is visible rather than silent.
    dropped_lod1_surfaces: usize,
    pending_appearances: Vec<(AppearanceBundle, Vec<SurfaceAppearance>)>,
    /// Maps original texture URI strings to relative output paths.
    uri_remap: HashMap<String, String>,
}

impl<W: Write> CityGmlXmlWriter<W> {
    pub fn new(inner: W, pretty: bool, srs_name: String) -> Self {
        let writer = if pretty {
            Writer::new_with_indent(inner, b' ', 2)
        } else {
            Writer::new(inner)
        };
        Self {
            writer,
            srs_name,
            id_counters: HashMap::new(),
            used_ids: HashSet::new(),
            dropped_lod1_surfaces: 0,
            pending_appearances: Vec::new(),
            uri_remap: HashMap::new(),
        }
    }

    /// How many `lod1MultiSurface` properties were dropped because the object
    /// also carried a `lod1Solid`. Read once the document is written.
    pub fn dropped_lod1_surfaces(&self) -> usize {
        self.dropped_lod1_surfaces
    }

    pub fn set_uri_remap(&mut self, remap: HashMap<String, String>) {
        self.uri_remap = remap;
    }

    fn generate_gml_id(&mut self, prefix: &str) -> String {
        let counter = self.id_counters.entry(prefix.to_string()).or_insert(0);
        *counter += 1;
        format!("{prefix}_{counter}")
    }

    /// Settle on a `gml:id` that is a legal `xs:ID` and unused in this document.
    ///
    /// `xs:ID` is an `NCName`, so it cannot start with a digit: a bare UUID is
    /// rejected by a validator roughly five times in eight. A candidate that is
    /// unusable, or already taken, falls back to a minted one.
    fn claim_gml_id(&mut self, candidate: Option<&str>, prefix: &str) -> String {
        let mut id = match candidate {
            Some(c) if is_ncname(c) => c.to_string(),
            Some(c) => format!("{prefix}_{}", sanitize_ncname(c)),
            None => self.generate_gml_id(prefix),
        };
        while !self.used_ids.insert(id.clone()) {
            id = self.generate_gml_id(prefix);
        }
        id
    }

    pub fn write_header(&mut self, envelope: Option<&BoundingEnvelope>) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut city_model = BytesStart::new("core:CityModel");
        for (prefix, uri) in CITYGML_2_NAMESPACES {
            city_model.push_attribute((*prefix, *uri));
        }
        city_model.push_attribute(("xsi:schemaLocation", CITYGML_2_SCHEMA_LOCATION));
        self.writer
            .write_event(Event::Start(city_model))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        if let Some(env) = envelope {
            self.write_bounded_by(env)?;
        }

        Ok(())
    }

    fn write_bounded_by(&mut self, envelope: &BoundingEnvelope) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:boundedBy")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut env_elem = BytesStart::new("gml:Envelope");
        env_elem.push_attribute(("srsName", self.srs_name.as_str()));
        env_elem.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(env_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Corners go through the same formatter as `gml:posList`, so the
        // envelope always reads in the same axis order as the geometry it
        // bounds, in whichever world is compiled.
        self.write_text_element("gml:lowerCorner", &format_pos_list(&[envelope.lower]))?;
        self.write_text_element("gml:upperCorner", &format_pos_list(&[envelope.upper]))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Envelope")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:boundedBy")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    pub fn write_city_object(
        &mut self,
        city_type: CityObjectType,
        geometries: Vec<GeometryEntry>,
        gml_id: Option<&str>,
        appearance: Option<&AppearanceBundle>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let element_name = city_type.element_name();
        let mut city_obj_elem = BytesStart::new(element_name);
        let obj_id = self.claim_gml_id(gml_id, city_type.id_prefix());
        city_obj_elem.push_attribute(("gml:id", obj_id.as_str()));
        self.writer
            .write_event(Event::Start(city_obj_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let need_appearance = appearance.is_some_and(|a| a.has_content());
        let mut surface_appearances: Vec<SurfaceAppearance> = Vec::new();

        // CityGML declares each class's properties as an `xs:sequence`, so arrival
        // order is not good enough: the elements must be emitted in schema order.
        // Cardinality is the other half of the same rule, so duplicates collapse
        // before the sort rather than being written as siblings.
        let mut merged = merge_duplicate_properties(geometries, city_type);
        self.dropped_lod1_surfaces += enforce_lod1_shell(&mut merged, city_type);
        let mut ordered: Vec<&GeometryEntry> = merged.iter().collect();
        let schema = SchemaSet::core().map_err(|e| {
            SinkError::CityGmlWriter(format!("the CityGML 2.0 schemas failed to load: {e}"))
        })?;
        // The class and namespace are the same for every entry, so look them up
        // once; the key is computed once per entry and the sort stays stable.
        let class = schema.class_for_element(&element_qname(city_type));
        let namespace = namespace_uri(city_type.namespace_prefix());
        ordered.sort_by_cached_key(|entry| {
            slot_position(class, namespace, &geometry_property_name(entry, city_type))
        });

        for entry in ordered {
            self.write_lod_geometry(city_type, entry, need_appearance, &mut surface_appearances)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(element_name)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Defer appearance to CityModel level (app:appearanceMember)
        if let Some(app) = appearance {
            if !surface_appearances.is_empty() {
                self.pending_appearances
                    .push((app.clone(), surface_appearances));
            }
        }

        Ok(())
    }

    fn write_lod_geometry(
        &mut self,
        city_type: CityObjectType,
        entry: &GeometryEntry,
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        let ns = city_type.namespace_prefix();
        let lod_elem = self.get_geometry_element_name(ns, entry, city_type);

        self.writer
            .write_event(Event::Start(BytesStart::new(&lod_elem)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        match &entry.element {
            GmlElement::Solid(solid) => {
                self.write_solid(solid, need_appearance, surface_appearances)?
            }
            GmlElement::MultiSolid { id, solids } => {
                self.write_multi_solid(id.as_deref(), solids, need_appearance, surface_appearances)?
            }
            GmlElement::MultiSurface { id, surfaces } => self.write_multi_surface(
                id.as_deref(),
                surfaces,
                need_appearance,
                surface_appearances,
            )?,
            GmlElement::MultiCurve { id, curves } => {
                self.write_multi_curve(id.as_deref(), curves)?
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(&lod_elem)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn get_geometry_element_name(
        &self,
        ns: &str,
        entry: &GeometryEntry,
        city_type: CityObjectType,
    ) -> String {
        format!("{}:{}", ns, geometry_property_name(entry, city_type))
    }

    fn write_solid(
        &mut self,
        solid: &GmlSolid,
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        let mut solid_elem = BytesStart::new("gml:Solid");
        if let Some(gml_id) = solid.id.as_deref() {
            solid_elem.push_attribute(("gml:id", gml_id));
        }
        solid_elem.push_attribute(("srsName", self.srs_name.as_str()));
        solid_elem.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(solid_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.write_shell(
            "gml:exterior",
            &solid.exterior,
            need_appearance,
            surface_appearances,
        )?;
        // One `gml:interior` per void. A solid converted from the legacy world
        // never has any: that reader discards interior shells at parse time.
        for shell in &solid.interiors {
            self.write_shell("gml:interior", shell, need_appearance, surface_appearances)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Solid")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    /// One shell of a solid: `wrapper` (`gml:exterior` or `gml:interior`)
    /// around the `gml:CompositeSurface` of the shell's faces.
    fn write_shell(
        &mut self,
        wrapper: &str,
        surfaces: &[GmlSurface],
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new(wrapper)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:CompositeSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for surface in surfaces {
            self.write_surface_member(surface, need_appearance, surface_appearances)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:CompositeSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new(wrapper)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_multi_solid(
        &mut self,
        id: Option<&str>,
        solids: &[GmlSolid],
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        let mut ms = BytesStart::new("gml:MultiSolid");
        if let Some(gml_id) = id {
            ms.push_attribute(("gml:id", gml_id));
        }
        ms.push_attribute(("srsName", self.srs_name.as_str()));
        ms.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(ms))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for solid in solids {
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:solidMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.write_solid(solid, need_appearance, surface_appearances)?;
            self.writer
                .write_event(Event::End(BytesEnd::new("gml:solidMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:MultiSolid")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_multi_surface(
        &mut self,
        id: Option<&str>,
        surfaces: &[GmlSurface],
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        let mut ms = BytesStart::new("gml:MultiSurface");
        if let Some(gml_id) = id {
            ms.push_attribute(("gml:id", gml_id));
        }
        ms.push_attribute(("srsName", self.srs_name.as_str()));
        ms.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(ms))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for surface in surfaces {
            self.write_surface_member(surface, need_appearance, surface_appearances)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:MultiSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    /// Write one `gml:surfaceMember`. If the surface has appearance data, generates a
    /// `gml:id` for the polygon (and ring IDs for textures) and records them in
    /// `surface_appearances` for the caller to write the `<app:appearance>` block.
    fn write_surface_member(
        &mut self,
        surface: &GmlSurface,
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:surfaceMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Generate a polygon gml:id only when appearance data references this surface
        let poly_id = if need_appearance
            && (surface.material_idx.is_some() || surface.texture_idx.is_some())
        {
            Some(self.claim_gml_id(None, "poly"))
        } else {
            surface
                .id
                .as_deref()
                .map(|id| self.claim_gml_id(Some(id), "poly"))
        };

        let mut polygon = BytesStart::new("gml:Polygon");
        if let Some(ref id) = poly_id {
            polygon.push_attribute(("gml:id", id.as_str()));
        }
        self.writer
            .write_event(Event::Start(polygon))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Exterior ring — gets a gml:id when texture is present
        let ext_ring_id = if surface.texture_idx.is_some() {
            poly_id.as_ref().map(|id| format!("{id}_e"))
        } else {
            None
        };
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.write_linear_ring(&surface.exterior, ext_ring_id.as_deref())?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Interior rings
        for (n, interior) in surface.interiors.iter().enumerate() {
            let int_ring_id = if surface.texture_idx.is_some() {
                poly_id.as_ref().map(|id| format!("{id}_i{n}"))
            } else {
                None
            };
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:interior")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.write_linear_ring(interior, int_ring_id.as_deref())?;
            self.writer
                .write_event(Event::End(BytesEnd::new("gml:interior")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Polygon")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:surfaceMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Record appearance info using the generated ID
        if let Some(id) = poly_id {
            if surface.material_idx.is_some() || surface.texture_idx.is_some() {
                surface_appearances.push(SurfaceAppearance {
                    surface_id: id,
                    material_idx: surface.material_idx,
                    texture_idx: surface.texture_idx,
                    uv_exterior: surface.uv_exterior.clone(),
                    uv_interiors: surface.uv_interiors.clone(),
                });
            }
        }

        Ok(())
    }

    fn write_linear_ring(
        &mut self,
        coords: &[[f64; 3]],
        ring_id: Option<&str>,
    ) -> Result<(), SinkError> {
        let mut ring = BytesStart::new("gml:LinearRing");
        if let Some(id) = ring_id {
            ring.push_attribute(("gml:id", id));
        }
        self.writer
            .write_event(Event::Start(ring))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.write_text_element("gml:posList", &format_pos_list(coords))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:LinearRing")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_multi_curve(
        &mut self,
        id: Option<&str>,
        curves: &[Vec<[f64; 3]>],
    ) -> Result<(), SinkError> {
        let mut mc = BytesStart::new("gml:MultiCurve");
        if let Some(gml_id) = id {
            mc.push_attribute(("gml:id", gml_id));
        }
        // Add srsName to geometry for proper CRS reference
        mc.push_attribute(("srsName", self.srs_name.as_str()));
        mc.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(mc))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for curve in curves {
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:curveMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:LineString")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            self.write_text_element("gml:posList", &format_pos_list(curve))?;

            self.writer
                .write_event(Event::End(BytesEnd::new("gml:LineString")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new("gml:curveMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:MultiCurve")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    /// Write `<app:appearance>` inside a city object, grouping surfaces by material/texture.
    fn write_appearance(
        &mut self,
        appearance: &AppearanceBundle,
        surface_appearances: &[SurfaceAppearance],
    ) -> Result<(), SinkError> {
        // Group surface IDs by material index
        let mut by_material: HashMap<u32, Vec<&str>> = HashMap::new();
        // Group targets by texture index: (surface_id, uv_exterior, uv_interiors)
        let mut by_texture: HashMap<u32, Vec<&SurfaceAppearance>> = HashMap::new();

        for sa in surface_appearances {
            if let Some(mat) = sa.material_idx {
                by_material.entry(mat).or_default().push(&sa.surface_id);
            }
            if let Some(tex) = sa.texture_idx {
                by_texture.entry(tex).or_default().push(sa);
            }
        }

        if by_material.is_empty() && by_texture.is_empty() {
            return Ok(());
        }

        self.writer
            .write_event(Event::Start(BytesStart::new("app:Appearance")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // The theme the converter actually selected, so downstream appearance
        // selection keeps working; only a world that records no theme name at
        // all falls back to `FALLBACK_THEME`.
        let theme = appearance.theme.as_deref().unwrap_or(FALLBACK_THEME);
        self.write_text_element("app:theme", theme)?;

        let mut sorted_materials: Vec<_> = by_material.into_iter().collect();
        sorted_materials.sort_by_key(|(k, _)| *k);
        for (mat_idx, ids) in &sorted_materials {
            if let Some(material) = appearance.materials.get(*mat_idx as usize) {
                self.write_x3d_material(material, ids)?;
            }
        }

        let mut sorted_textures: Vec<_> = by_texture.into_iter().collect();
        sorted_textures.sort_by_key(|(k, _)| *k);
        for (tex_idx, targets) in &sorted_textures {
            if let Some(texture) = appearance.textures.get(*tex_idx as usize) {
                self.write_parameterized_texture(texture, targets)?;
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("app:Appearance")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_x3d_material(
        &mut self,
        material: &X3DMaterial,
        target_ids: &[&str],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Start(BytesStart::new("app:X3DMaterial")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.write_text_element(
            "app:ambientIntensity",
            &material.ambient_intensity.to_string(),
        )?;
        let c = &material.diffuse_color;
        self.write_text_element("app:diffuseColor", &format!("{} {} {}", c.r, c.g, c.b))?;
        let c = &material.specular_color;
        self.write_text_element("app:specularColor", &format!("{} {} {}", c.r, c.g, c.b))?;
        for id in target_ids {
            self.write_text_element("app:target", &format!("#{id}"))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("app:X3DMaterial")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_parameterized_texture(
        &mut self,
        texture: &GmlTexture,
        targets: &[&SurfaceAppearance],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Start(BytesStart::new("app:ParameterizedTexture")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Keyed on what the texture was staged under, not on its URI: an
        // in-memory raster has no URI to key by. For a URI-backed one the key
        // *is* the URI string.
        let image_uri = self
            .uri_remap
            .get(texture.key.as_str())
            .cloned()
            .unwrap_or_else(|| texture.uri.clone());
        self.write_text_element("app:imageURI", image_uri.as_str())?;
        self.write_text_element("app:mimeType", mime_type_from_uri(image_uri.as_str()))?;

        for sa in targets {
            let mut target_elem = BytesStart::new("app:target");
            target_elem.push_attribute(("uri", format!("#{}", sa.surface_id).as_str()));
            self.writer
                .write_event(Event::Start(target_elem))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            self.writer
                .write_event(Event::Start(BytesStart::new("app:TexCoordList")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            // Exterior ring UV
            let ext_ring_id = format!("#{}_e", sa.surface_id);
            let mut tex_coords = BytesStart::new("app:textureCoordinates");
            tex_coords.push_attribute(("ring", ext_ring_id.as_str()));
            self.writer
                .write_event(Event::Start(tex_coords))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Text(BytesText::new(&format_uv_coords(
                    &sa.uv_exterior,
                ))))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new("app:textureCoordinates")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            // Interior rings UV
            for (n, uv_int) in sa.uv_interiors.iter().enumerate() {
                let int_ring_id = format!("#{}_i{n}", sa.surface_id);
                let mut tex_coords = BytesStart::new("app:textureCoordinates");
                tex_coords.push_attribute(("ring", int_ring_id.as_str()));
                self.writer
                    .write_event(Event::Start(tex_coords))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer
                    .write_event(Event::Text(BytesText::new(&format_uv_coords(uv_int))))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer
                    .write_event(Event::End(BytesEnd::new("app:textureCoordinates")))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }

            self.writer
                .write_event(Event::End(BytesEnd::new("app:TexCoordList")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new("app:target")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("app:ParameterizedTexture")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_text_element(&mut self, tag: &str, text: &str) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new(tag)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }

    pub fn flush_appearances(&mut self) -> Result<(), SinkError> {
        let pending = std::mem::take(&mut self.pending_appearances);
        for (bundle, surfaces) in &pending {
            self.writer
                .write_event(Event::Start(BytesStart::new("app:appearanceMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.write_appearance(bundle, surfaces)?;
            self.writer
                .write_event(Event::End(BytesEnd::new("app:appearanceMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }
        Ok(())
    }

    pub fn write_footer(&mut self) -> Result<(), SinkError> {
        self.flush_appearances()?;
        self.writer
            .write_event(Event::End(BytesEnd::new("core:CityModel")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }
}

/// Whether `s` is an XML `NCName`, which is what `xs:ID` requires.
pub(super) fn is_ncname(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// Replace whatever an `NCName` cannot carry, so a prefixed id stays legal.
fn sanitize_ncname(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '_' | '-' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// The local name of the property this geometry fills.
///
/// A source document's own property name wins, because the LOD digit cannot
/// recover it: `lod0FootPrint` and `lod0RoofEdge` share both LOD and GML family.
/// Everything else is synthesised from the class, the LOD and the family.
fn geometry_property_name(entry: &GeometryEntry, city_type: CityObjectType) -> String {
    if let Some(property) = &entry.property {
        return property.clone();
    }
    if city_type == CityObjectType::GenericCityObject {
        return format!("lod{}Geometry", entry.lod);
    }
    let family = match &entry.element {
        GmlElement::Solid(_) => "Solid",
        GmlElement::MultiSolid { .. } => "MultiSolid",
        GmlElement::MultiSurface { .. } => "MultiSurface",
        GmlElement::MultiCurve { .. } => "MultiCurve",
    };
    format!("lod{}{}", entry.lod, family)
}

/// Collapse entries that resolve to the same property name into one element.
///
/// Every `lodNXxx` property is declared `minOccurs="0"` with no `maxOccurs` in
/// the CityGML schemas, so it may appear at most once; `boundedBy` is the only
/// repeating one. Nested objects are currently flattened onto their parent, so a
/// building with two `bldg:WallSurface` children arrives as two entries both
/// naming `lod2MultiSurface`. Writing both produces a document no validator
/// accepts, so they merge into a single multi-geometry here.
///
/// This keeps the geometry while losing which boundary surface each piece came
/// from. That distinction is already lost upstream by the flattening, so nothing
/// survives the merge that would have survived without it.
fn merge_duplicate_properties(
    entries: Vec<GeometryEntry>,
    city_type: CityObjectType,
) -> Vec<GeometryEntry> {
    let mut merged: Vec<GeometryEntry> = Vec::with_capacity(entries.len());
    for entry in entries {
        let name = geometry_property_name(&entry, city_type);
        let unfolded = match merged
            .iter_mut()
            .find(|held| geometry_property_name(held, city_type) == name)
        {
            Some(held) => absorb(held, entry),
            None => Some(entry),
        };
        // Nothing that cannot be folded is thrown away: it stays a separate
        // entry, so the document keeps all of its geometry, and the schema gate
        // still sees the problem.
        if let Some(entry) = unfolded {
            merged.push(entry);
        }
    }
    merged
}

/// Whether this class inherits the LOD1 exclusive-shell rule.
///
/// CityGML 2.0 states it identically for `_AbstractBuilding` (§10.3.4),
/// `_AbstractTunnel` and `_AbstractBridge`: at LOD1 the volumetric and surface
/// parts of the exterior shell are identical, so either `lodNSolid` or
/// `lodNMultiSurface` must be used, but not both, and from LOD2 the two may be
/// modelled individually and complementary. `WaterBody` declares the same pair
/// at LOD1 and carries no such restriction, so it is deliberately absent here.
fn has_exclusive_lod1_shell(city_type: CityObjectType) -> bool {
    matches!(
        city_type,
        CityObjectType::Building
            | CityObjectType::BuildingPart
            | CityObjectType::Bridge
            | CityObjectType::BridgePart
            | CityObjectType::Tunnel
            | CityObjectType::TunnelPart
    )
}

/// Drop `lod1MultiSurface` when `lod1Solid` is also present, and report it.
///
/// The XSD declares the two as independent optional elements, so a document
/// carrying both validates and the CI schema gate cannot catch this. The rule
/// exists only in the specification text, which is why it is enforced here.
///
/// The solid wins because `building.xsd` says the multi-surface form is the one
/// to use when the geometry is not a topologically clean solid, making the solid
/// the primary representation. Since the spec also defines the two as identical
/// at LOD1, the dropped surfaces are redundant rather than additional.
fn enforce_lod1_shell(entries: &mut Vec<GeometryEntry>, city_type: CityObjectType) -> usize {
    if !has_exclusive_lod1_shell(city_type) {
        return 0;
    }
    let names: Vec<String> = entries
        .iter()
        .map(|entry| geometry_property_name(entry, city_type))
        .collect();
    if !names.iter().any(|name| name == "lod1Solid") {
        return 0;
    }
    let before = entries.len();
    let mut index = 0;
    entries.retain(|_| {
        let keep = names[index] != "lod1MultiSurface";
        index += 1;
        keep
    });
    before - entries.len()
}

/// Fold `extra`'s geometry into `held`, which already occupies that property.
/// Hands `extra` back if it cannot, which is whenever the GML families differ:
/// a second `gml:Solid` has nowhere to go inside a `SolidPropertyType` property.
fn absorb(held: &mut GeometryEntry, extra: GeometryEntry) -> Option<GeometryEntry> {
    match (&mut held.element, extra.element) {
        (
            GmlElement::MultiSurface { surfaces, .. },
            GmlElement::MultiSurface { surfaces: more, .. },
        ) => surfaces.extend(more),
        (GmlElement::MultiCurve { curves, .. }, GmlElement::MultiCurve { curves: more, .. }) => {
            curves.extend(more)
        }
        (GmlElement::MultiSolid { solids, .. }, GmlElement::MultiSolid { solids: more, .. }) => {
            solids.extend(more)
        }
        (_, element) => {
            return Some(GeometryEntry {
                lod: extra.lod,
                property: extra.property,
                element,
            })
        }
    }
    None
}

/// Where the property `local` sits in `city_type`'s content model, or
/// `usize::MAX` for a name no slot accepts. A property the schema does not
/// declare sorts last rather than being dropped, so invalid input stays visible
/// to the schema gate instead of vanishing, and it keeps its arrival order
/// under the stable sort.
#[cfg(test)]
fn property_position(schema: &SchemaSet, city_type: CityObjectType, local: &str) -> usize {
    slot_position(
        schema.class_for_element(&element_qname(city_type)),
        namespace_uri(city_type.namespace_prefix()),
        local,
    )
}

fn slot_position(class: Option<&ClassModel>, namespace: &str, local: &str) -> usize {
    class
        .and_then(|class| class.slot_index(&QName::new(namespace, local)))
        .unwrap_or(usize::MAX)
}

/// The element `city_type` is written as, fully qualified.
fn element_qname(city_type: CityObjectType) -> QName {
    let (prefix, local) = city_type
        .element_name()
        .split_once(':')
        .expect("every element name carries a prefix");
    QName::new(namespace_uri(prefix), local)
}

/// The namespace the writer binds `prefix` to, or `None` for a prefix it
/// never declares. `xmlns` itself is not a prefix a property can use.
pub(super) fn namespace_for_prefix(prefix: &str) -> Option<&'static str> {
    let attribute = format!("xmlns:{prefix}");
    CITYGML_2_NAMESPACES
        .iter()
        .chain(LOCALLY_DECLARED_NAMESPACES)
        .find(|(name, _)| *name == attribute)
        .map(|(_, uri)| *uri)
}

/// The namespace the writer binds `prefix` to in `CITYGML_2_NAMESPACES`.
fn namespace_uri(prefix: &str) -> &'static str {
    namespace_for_prefix(prefix).expect("every class prefix is declared in CITYGML_2_NAMESPACES")
}

fn format_uv_coords(uvs: &[[f64; 2]]) -> String {
    uvs.iter()
        .map(|uv| format!("{} {}", uv[0], uv[1]))
        .collect::<Vec<_>>()
        .join(" ")
}

fn mime_type_from_uri(uri: &str) -> &'static str {
    let lower = uri.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_types::material::X3DMaterial;

    use super::*;

    const SRS: &str = "http://www.opengis.net/def/crs/EPSG/0/6697";

    use reearth_flow_citygml::schema::SchemaSet;

    const ALL_CITY_TYPES: [CityObjectType; 17] = [
        CityObjectType::Building,
        CityObjectType::BuildingPart,
        CityObjectType::Road,
        CityObjectType::Railway,
        CityObjectType::Track,
        CityObjectType::Square,
        CityObjectType::Bridge,
        CityObjectType::BridgePart,
        CityObjectType::Tunnel,
        CityObjectType::TunnelPart,
        CityObjectType::WaterBody,
        CityObjectType::LandUse,
        CityObjectType::SolitaryVegetationObject,
        CityObjectType::PlantCover,
        CityObjectType::CityFurniture,
        CityObjectType::ReliefFeature,
        CityObjectType::GenericCityObject,
    ];

    /// Every name `geometry_property_name` can produce for a class.
    fn geometry_names(city_type: CityObjectType) -> Vec<String> {
        let families: &[&str] = if city_type == CityObjectType::GenericCityObject {
            &["Geometry"]
        } else {
            &["Solid", "MultiSolid", "MultiSurface", "MultiCurve"]
        };
        (0..=4)
            .flat_map(|lod| families.iter().map(move |f| format!("lod{lod}{f}")))
            .collect()
    }

    fn frozen_order() -> HashMap<String, Vec<String>> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../citygml/tests/data/content_model_2_0.txt");
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| {
                let (class, props) = line.split_once('\t').unwrap();
                (
                    class.to_owned(),
                    props.split(' ').map(str::to_owned).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn every_city_type_has_a_class_model() {
        let schema = SchemaSet::core().unwrap();
        for city_type in ALL_CITY_TYPES {
            assert!(
                schema
                    .class_for_element(&element_qname(city_type))
                    .is_some(),
                "{city_type:?}"
            );
        }
    }

    /// The switch must not reorder anything: a name the old table listed keeps
    /// its relative order, and a name it did not list still sorts last.
    #[test]
    fn geometry_properties_order_exactly_as_the_generated_table_did() {
        let schema = SchemaSet::core().unwrap();
        let frozen = frozen_order();
        for city_type in ALL_CITY_TYPES {
            let old = &frozen[&format!("{city_type:?}")];
            let mut listed: Vec<(usize, usize)> = Vec::new();
            // The names the writer builds, plus every `lod*` property the old
            // table listed for the class: converters supply some of those
            // (`lodNGeometry`, terrain intersections, `lod0FootPrint`, ...).
            let mut names = geometry_names(city_type);
            names.extend(old.iter().filter(|p| p.starts_with("lod")).cloned());
            names.sort();
            names.dedup();
            for name in names {
                let new = property_position(schema, city_type, &name);
                match old.iter().position(|p| *p == name) {
                    Some(old_index) => {
                        assert_ne!(new, usize::MAX, "{city_type:?} {name} lost its slot");
                        listed.push((old_index, new));
                    }
                    None => assert_eq!(new, usize::MAX, "{city_type:?} {name} gained a slot"),
                }
            }
            listed.sort();
            assert!(
                listed.windows(2).all(|w| w[0].1 < w[1].1),
                "{city_type:?} order changed: {listed:?}"
            );
        }
    }

    /// Coordinates as the compiled world stores them, chosen so both formatters
    /// emit the *same* `posList`. That equality is the axis-order invariant, and
    /// it is why every expected XML string below is written once.
    #[cfg(not(feature = "new-geometry"))]
    mod fixture {
        /// Exterior: 3 coords → posList "35 139 0 35 139.1 0 35.1 139 0"
        /// (integer-valued floats drop the decimal).
        pub fn triangle() -> Vec<[f64; 3]> {
            vec![[139.0, 35.0, 0.0], [139.1, 35.0, 0.0], [139.0, 35.1, 0.0]]
        }

        /// Offset from `triangle()` so the two are distinguishable in an expected
        /// posList: "35.01 139.01 1 35.01 139.02 1 35.02 139.01 1".
        pub fn offset_triangle() -> Vec<[f64; 3]> {
            vec![
                [139.01, 35.01, 1.0],
                [139.02, 35.01, 1.0],
                [139.01, 35.02, 1.0],
            ]
        }
    }

    #[cfg(feature = "new-geometry")]
    mod fixture {
        /// The same triangle, stored in EPSG:6697's declared order (lat, lon,
        /// height), which the identity formatter writes unchanged.
        pub fn triangle() -> Vec<[f64; 3]> {
            vec![[35.0, 139.0, 0.0], [35.0, 139.1, 0.0], [35.1, 139.0, 0.0]]
        }

        pub fn offset_triangle() -> Vec<[f64; 3]> {
            vec![
                [35.01, 139.01, 1.0],
                [35.01, 139.02, 1.0],
                [35.02, 139.01, 1.0],
            ]
        }
    }

    use fixture::{offset_triangle, triangle};

    /// A geometry-only surface: no material, no texture, so the writer mints no
    /// `gml:id` for it and the expected XML stays about the nesting.
    fn plain_surface(exterior: Vec<[f64; 3]>) -> GmlSurface {
        GmlSurface {
            id: None,
            exterior,
            interiors: vec![],
            material_idx: None,
            texture_idx: None,
            uv_exterior: vec![],
            uv_interiors: vec![],
        }
    }

    /// Write several geometry entries as one Building city object and return the
    /// XML. Separate from `write_entry` because cardinality only shows up with
    /// more than one entry in hand.
    fn write_entries(entries: Vec<GeometryEntry>) -> String {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.write_city_object(CityObjectType::Building, entries, Some("obj-001"), None)
            .unwrap();
        w.flush_appearances().unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// A building whose LOD-2 geometry came from two boundary surfaces, which is
    /// what the reader hands over for a `bldg:Building` carrying two
    /// `bldg:WallSurface` children: one member each, both naming the same source
    /// property.
    ///
    /// `bldg:lod2MultiSurface` is declared `minOccurs="0"` with no `maxOccurs` in
    /// `building.xsd`, so it may appear **once**. Only `bldg:boundedBy` is
    /// `maxOccurs="unbounded"`. Emitting the property twice is therefore invalid
    /// against the schema, and no validator will accept the document.
    #[test]
    fn duplicate_lod_property_is_merged_into_one_element() {
        let entries = vec![
            GeometryEntry {
                lod: 2,
                property: Some("lod2MultiSurface".to_string()),
                element: GmlElement::MultiSurface {
                    id: None,
                    surfaces: vec![plain_surface(triangle())],
                },
            },
            GeometryEntry {
                lod: 2,
                property: Some("lod2MultiSurface".to_string()),
                element: GmlElement::MultiSurface {
                    id: None,
                    surfaces: vec![plain_surface(offset_triangle())],
                },
            },
        ];

        let xml = write_entries(entries);

        assert_eq!(
            xml.matches("<bldg:lod2MultiSurface>").count(),
            1,
            "the property may appear at most once, got {} occurrences:\n{xml}",
            xml.matches("<bldg:lod2MultiSurface>").count()
        );
        assert_eq!(
            xml.matches("<gml:surfaceMember>").count(),
            2,
            "merging must keep both surfaces, not drop one:\n{xml}"
        );
    }

    /// Curves collide the same way surfaces do: `lod0RoofEdge` is one property,
    /// and two source objects carrying it arrive as two entries. Merging must
    /// keep both curves rather than letting the second one fall on the floor.
    #[test]
    fn duplicate_curve_property_keeps_every_curve() {
        let entries = vec![
            GeometryEntry {
                lod: 0,
                property: Some("lod0RoofEdge".to_string()),
                element: GmlElement::MultiCurve {
                    id: None,
                    curves: vec![triangle()],
                },
            },
            GeometryEntry {
                lod: 0,
                property: Some("lod0RoofEdge".to_string()),
                element: GmlElement::MultiCurve {
                    id: None,
                    curves: vec![offset_triangle()],
                },
            },
        ];

        let xml = write_entries(entries);

        assert_eq!(
            xml.matches("<bldg:lod0RoofEdge>").count(),
            1,
            "the property may appear at most once:\n{xml}"
        );
        assert_eq!(
            xml.matches("<gml:curveMember>").count(),
            2,
            "merging must keep both curves, not drop one:\n{xml}"
        );
    }

    /// Two solids cannot share one `lod2Solid`, because `gml:SolidPropertyType`
    /// holds exactly one `gml:Solid`. There is no valid single-property output
    /// for this input, so the writer keeps both rather than silently discarding
    /// one: the document stays invalid and the schema gate still reports it,
    /// which is the lesser of the two failures.
    ///
    /// The real fix is nesting, since two solids at one LOD means two objects.
    /// Until that lands this is a known limitation, pinned here so a future
    /// change to drop-on-collide cannot slip in unnoticed.
    #[test]
    fn unmergeable_solids_are_kept_rather_than_dropped() {
        let solid = |exterior: Vec<[f64; 3]>| GmlSolid {
            id: None,
            exterior: vec![plain_surface(exterior)],
            interiors: vec![],
        };
        let entries = vec![
            GeometryEntry {
                lod: 2,
                property: Some("lod2Solid".to_string()),
                element: GmlElement::Solid(solid(triangle())),
            },
            GeometryEntry {
                lod: 2,
                property: Some("lod2Solid".to_string()),
                element: GmlElement::Solid(solid(offset_triangle())),
            },
        ];

        let xml = write_entries(entries);

        assert_eq!(
            xml.matches("<gml:Solid").count(),
            2,
            "neither solid may be discarded:\n{xml}"
        );
    }

    /// Minted ids are numbered per prefix, so a city object and the polygons
    /// inside it do not share one running count.
    ///
    /// This is what lets a feature with no source `gml:id` take a deterministic
    /// id without renumbering every `poly_N` beneath it, and a deterministic id
    /// is what lets the test framework compare ids instead of masking them.
    #[test]
    fn minted_ids_are_numbered_per_prefix() {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());

        assert_eq!(w.claim_gml_id(None, "bldg"), "bldg_1");
        assert_eq!(w.claim_gml_id(None, "poly"), "poly_1");
        assert_eq!(w.claim_gml_id(None, "poly"), "poly_2");
        assert_eq!(w.claim_gml_id(None, "bldg"), "bldg_2");
    }

    fn solid_of(exterior: Vec<[f64; 3]>) -> GmlSolid {
        GmlSolid {
            id: None,
            exterior: vec![plain_surface(exterior)],
            interiors: vec![],
        }
    }

    fn shell_entries(lod: u8) -> Vec<GeometryEntry> {
        vec![
            GeometryEntry {
                lod,
                property: Some(format!("lod{lod}Solid")),
                element: GmlElement::Solid(solid_of(triangle())),
            },
            GeometryEntry {
                lod,
                property: Some(format!("lod{lod}MultiSurface")),
                element: GmlElement::MultiSurface {
                    id: None,
                    surfaces: vec![plain_surface(offset_triangle())],
                },
            },
        ]
    }

    /// CityGML 2.0 §10.3.4, conformance requirement 3: at LOD1 the volumetric and
    /// surface parts of the exterior shell are identical, so either `lod1Solid`
    /// or `lod1MultiSurface` must be used, **but not both**.
    ///
    /// The XSD declares them as two independent optional elements, so a document
    /// carrying both still passes schema validation and the CI gate cannot see
    /// it. The solid is kept because `building.xsd` says the multi-surface form
    /// is the one to use when the geometry is not a topologically clean solid,
    /// which makes the solid the primary representation.
    #[test]
    fn lod1_shell_keeps_the_solid_and_drops_the_multisurface() {
        let xml = write_entries(shell_entries(1));

        assert_eq!(
            xml.matches("<bldg:lod1Solid>").count(),
            1,
            "the solid is the primary form and must survive:\n{xml}"
        );
        assert_eq!(
            xml.matches("<bldg:lod1MultiSurface>").count(),
            0,
            "LOD1 may carry only one of the two:\n{xml}"
        );
    }

    /// The same spec paragraph: "Starting from LOD2, both properties may be
    /// modelled individually and complementary." So the LOD1 rule must not be
    /// generalised into a blanket exclusion.
    #[test]
    fn lod2_may_carry_both_solid_and_multisurface() {
        let xml = write_entries(shell_entries(2));

        assert_eq!(xml.matches("<bldg:lod2Solid>").count(), 1, "{xml}");
        assert_eq!(
            xml.matches("<bldg:lod2MultiSurface>").count(),
            1,
            "from LOD2 the two are complementary, not exclusive:\n{xml}"
        );
    }

    /// Write one geometry entry as a Building city object and return the XML.
    fn write_entry(entry: GeometryEntry, appearance: Option<&AppearanceBundle>) -> String {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.write_city_object(
            CityObjectType::Building,
            vec![entry],
            Some("obj-001"),
            appearance,
        )
        .unwrap();
        w.flush_appearances().unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Write a single LOD-2 MultiSurface Building and return the XML output.
    fn write_building(surfaces: Vec<GmlSurface>, appearance: Option<&AppearanceBundle>) -> String {
        write_entry(
            GeometryEntry {
                lod: 2,
                property: None,
                element: GmlElement::MultiSurface { id: None, surfaces },
            },
            appearance,
        )
    }

    // X3DMaterial

    #[test]
    fn test_write_city_object_x3d_material() {
        // Surface references material index 0; no texture.
        // id_counter starts at 0 → first generate_gml_id("poly") → "poly_1".
        // LinearRing gets no gml:id (no texture).
        let surface = GmlSurface {
            id: None,
            exterior: triangle(),
            interiors: vec![],
            material_idx: Some(0),
            texture_idx: None,
            uv_exterior: vec![],
            uv_interiors: vec![],
        };
        let appearance = AppearanceBundle {
            theme: None,
            // X3DMaterial::default(): diffuse=0.7/0.7/0.7, specular=0.04/0.04/0.04, ambient=0.9
            materials: vec![X3DMaterial::default()],
            textures: vec![],
        };

        let xml = write_building(vec![surface], Some(&appearance));

        let expected = concat!(
            r#"<core:cityObjectMember>"#,
            r#"<bldg:Building gml:id="obj-001">"#,
            r#"<bldg:lod2MultiSurface>"#,
            r#"<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon gml:id="poly_1">"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing>"#, // no gml:id — material only, not texture
            r#"<gml:posList>35 139 0 35 139.1 0 35.1 139 0</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:MultiSurface>"#,
            r#"</bldg:lod2MultiSurface>"#,
            r#"</bldg:Building>"#,
            r#"</core:cityObjectMember>"#,
            // Appearance is now a top-level app:appearanceMember
            r#"<app:appearanceMember>"#,
            r#"<app:Appearance>"#,
            r#"<app:theme>rgbTexture</app:theme>"#,
            r#"<app:surfaceDataMember><app:X3DMaterial>"#,
            r#"<app:ambientIntensity>0.9</app:ambientIntensity>"#,
            r#"<app:diffuseColor>0.7 0.7 0.7</app:diffuseColor>"#,
            r#"<app:specularColor>0.04 0.04 0.04</app:specularColor>"#,
            r#"<app:target>#poly_1</app:target>"#,
            r#"</app:X3DMaterial></app:surfaceDataMember>"#,
            r#"</app:Appearance>"#,
            r#"</app:appearanceMember>"#,
        );
        assert_eq!(xml, expected);
    }

    // ParameterizedTexture

    #[test]
    fn test_write_city_object_parameterized_texture() {
        // Surface references texture index 0 with UV coords.
        // Polygon gets gml:id="poly_1", exterior LinearRing gets gml:id="poly_1_e".
        // UV: [[0,0],[1,0],[0.5,1]] → "0 0 1 0 0.5 1"
        let surface = GmlSurface {
            id: None,
            exterior: triangle(),
            interiors: vec![],
            material_idx: None,
            texture_idx: Some(0),
            uv_exterior: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            uv_interiors: vec![],
        };
        let appearance = AppearanceBundle {
            theme: None,
            materials: vec![],
            textures: vec![GmlTexture {
                key: "file:///textures/wall.jpg".to_string(),
                uri: "file:///textures/wall.jpg".to_string(),
            }],
        };

        let xml = write_building(vec![surface], Some(&appearance));

        let expected = concat!(
            r#"<core:cityObjectMember>"#,
            r#"<bldg:Building gml:id="obj-001">"#,
            r#"<bldg:lod2MultiSurface>"#,
            r#"<gml:MultiSurface srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon gml:id="poly_1">"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing gml:id="poly_1_e">"#, // ring ID for texture coord reference
            r#"<gml:posList>35 139 0 35 139.1 0 35.1 139 0</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:MultiSurface>"#,
            r#"</bldg:lod2MultiSurface>"#,
            r#"</bldg:Building>"#,
            r#"</core:cityObjectMember>"#,
            // Appearance is now a top-level app:appearanceMember
            r#"<app:appearanceMember>"#,
            r#"<app:Appearance>"#,
            r#"<app:theme>rgbTexture</app:theme>"#,
            r#"<app:surfaceDataMember><app:ParameterizedTexture>"#,
            r#"<app:imageURI>file:///textures/wall.jpg</app:imageURI>"#,
            r#"<app:mimeType>image/jpeg</app:mimeType>"#,
            r##"<app:target uri="#poly_1">"##,
            r#"<app:TexCoordList>"#,
            r##"<app:textureCoordinates ring="#poly_1_e">0 0 1 0 0.5 1</app:textureCoordinates>"##,
            r#"</app:TexCoordList>"#,
            r#"</app:target>"#,
            r#"</app:ParameterizedTexture></app:surfaceDataMember>"#,
            r#"</app:Appearance>"#,
            r#"</app:appearanceMember>"#,
        );
        assert_eq!(xml, expected);
    }

    /// A named theme wins over the fallback theme: a wrong `app:theme` breaks
    /// appearance selection for anything reading the output back.
    #[test]
    fn a_named_theme_is_written_instead_of_the_fallback_literal() {
        let surface = GmlSurface {
            id: None,
            exterior: triangle(),
            interiors: vec![],
            material_idx: Some(0),
            texture_idx: None,
            uv_exterior: vec![],
            uv_interiors: vec![],
        };
        let appearance = AppearanceBundle {
            theme: Some("iurTexture".to_string()),
            materials: vec![X3DMaterial::default()],
            textures: vec![],
        };

        let xml = write_building(vec![surface], Some(&appearance));

        assert!(
            xml.contains("<app:theme>iurTexture</app:theme>"),
            "expected the selected theme, got: {xml}"
        );
        assert!(!xml.contains("rgbTexture"), "{xml}");
    }

    /// `app:imageURI` is rewritten by the staging key, which for an in-memory
    /// raster is not a URI at all. The fallback URI is only what a texture that
    /// was never staged keeps.
    #[test]
    fn the_image_uri_is_rewritten_by_the_staging_key() {
        let surface = GmlSurface {
            id: None,
            exterior: triangle(),
            interiors: vec![],
            material_idx: None,
            texture_idx: Some(0),
            uv_exterior: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
            uv_interiors: vec![],
        };
        let appearance = AppearanceBundle {
            theme: Some("rgbTexture".to_string()),
            materials: vec![],
            textures: vec![GmlTexture {
                key: "inline:0123456789abcdef".to_string(),
                uri: "0123456789abcdef.png".to_string(),
            }],
        };

        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.set_uri_remap(HashMap::from([(
            "inline:0123456789abcdef".to_string(),
            "city_appearance/0123456789abcdef.png".to_string(),
        )]));
        w.write_city_object(
            CityObjectType::Building,
            vec![GeometryEntry {
                lod: 2,
                property: None,
                element: GmlElement::MultiSurface {
                    id: None,
                    surfaces: vec![surface],
                },
            }],
            Some("obj-001"),
            Some(&appearance),
        )
        .unwrap();
        w.flush_appearances().unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(
            xml.contains(
                "<app:imageURI>city_appearance/0123456789abcdef.png</app:imageURI>\
                 <app:mimeType>image/png</app:mimeType>"
            ),
            "{xml}"
        );
    }

    // Solid shells

    /// A void shell is a second `gml:CompositeSurface` under `gml:interior`,
    /// sibling to the exterior one — the nesting the legacy build could never
    /// emit, because its reader discards interior shells at parse time.
    #[test]
    fn test_write_solid_with_a_void_shell() {
        let entry = GeometryEntry {
            lod: 1,
            property: None,
            element: GmlElement::Solid(GmlSolid {
                id: Some("solid-1".to_string()),
                exterior: vec![plain_surface(triangle())],
                interiors: vec![vec![plain_surface(offset_triangle())]],
            }),
        };

        let xml = write_entry(entry, None);

        let expected = concat!(
            r#"<core:cityObjectMember>"#,
            r#"<bldg:Building gml:id="obj-001">"#,
            r#"<bldg:lod1Solid>"#,
            r#"<gml:Solid gml:id="solid-1" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:exterior>"#,
            r#"<gml:CompositeSurface>"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon>"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing>"#,
            r#"<gml:posList>35 139 0 35 139.1 0 35.1 139 0</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:CompositeSurface>"#,
            r#"</gml:exterior>"#,
            // The void, as its own composite surface.
            r#"<gml:interior>"#,
            r#"<gml:CompositeSurface>"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon>"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing>"#,
            r#"<gml:posList>35.01 139.01 1 35.01 139.02 1 35.02 139.01 1</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:CompositeSurface>"#,
            r#"</gml:interior>"#,
            r#"</gml:Solid>"#,
            r#"</bldg:lod1Solid>"#,
            r#"</bldg:Building>"#,
            r#"</core:cityObjectMember>"#,
        );
        assert_eq!(xml, expected);
    }

    // MultiSolid

    /// Each solid of a `gml:MultiSolid` is its own `gml:solidMember`, and the
    /// retained source property name is the wrapper verbatim.
    #[test]
    fn test_write_multi_solid_of_two_solids() {
        let entry = GeometryEntry {
            lod: 2,
            property: Some("lod2MultiSolid".to_string()),
            element: GmlElement::MultiSolid {
                id: Some("msolid-1".to_string()),
                solids: vec![
                    GmlSolid {
                        id: Some("solid-a".to_string()),
                        exterior: vec![plain_surface(triangle())],
                        interiors: vec![],
                    },
                    GmlSolid {
                        id: Some("solid-b".to_string()),
                        exterior: vec![plain_surface(offset_triangle())],
                        interiors: vec![],
                    },
                ],
            },
        };

        let xml = write_entry(entry, None);

        let expected = concat!(
            r#"<core:cityObjectMember>"#,
            r#"<bldg:Building gml:id="obj-001">"#,
            r#"<bldg:lod2MultiSolid>"#,
            r#"<gml:MultiSolid gml:id="msolid-1" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:solidMember>"#,
            r#"<gml:Solid gml:id="solid-a" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:exterior>"#,
            r#"<gml:CompositeSurface>"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon>"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing>"#,
            r#"<gml:posList>35 139 0 35 139.1 0 35.1 139 0</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:CompositeSurface>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Solid>"#,
            r#"</gml:solidMember>"#,
            r#"<gml:solidMember>"#,
            r#"<gml:Solid gml:id="solid-b" srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:exterior>"#,
            r#"<gml:CompositeSurface>"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon>"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing>"#,
            r#"<gml:posList>35.01 139.01 1 35.01 139.02 1 35.02 139.01 1</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:CompositeSurface>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Solid>"#,
            r#"</gml:solidMember>"#,
            r#"</gml:MultiSolid>"#,
            r#"</bldg:lod2MultiSolid>"#,
            r#"</bldg:Building>"#,
            r#"</core:cityObjectMember>"#,
        );
        assert_eq!(xml, expected);
    }

    /// With no retained property name the wrapper falls back to the LOD and the
    /// GML family, which for a `MultiSolid` has to name `MultiSolid` — not the
    /// `Solid` its members are.
    #[test]
    fn test_multi_solid_property_name_falls_back_to_the_family() {
        let entry = GeometryEntry {
            lod: 3,
            property: None,
            element: GmlElement::MultiSolid {
                id: None,
                solids: vec![GmlSolid {
                    id: None,
                    exterior: vec![plain_surface(triangle())],
                    interiors: vec![],
                }],
            },
        };

        let xml = write_entry(entry, None);

        let expected = concat!(
            r#"<core:cityObjectMember>"#,
            r#"<bldg:Building gml:id="obj-001">"#,
            r#"<bldg:lod3MultiSolid>"#,
            r#"<gml:MultiSolid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:solidMember>"#,
            r#"<gml:Solid srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">"#,
            r#"<gml:exterior>"#,
            r#"<gml:CompositeSurface>"#,
            r#"<gml:surfaceMember>"#,
            r#"<gml:Polygon>"#,
            r#"<gml:exterior>"#,
            r#"<gml:LinearRing>"#,
            r#"<gml:posList>35 139 0 35 139.1 0 35.1 139 0</gml:posList>"#,
            r#"</gml:LinearRing>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Polygon>"#,
            r#"</gml:surfaceMember>"#,
            r#"</gml:CompositeSurface>"#,
            r#"</gml:exterior>"#,
            r#"</gml:Solid>"#,
            r#"</gml:solidMember>"#,
            r#"</gml:MultiSolid>"#,
            r#"</bldg:lod3MultiSolid>"#,
            r#"</bldg:Building>"#,
            r#"</core:cityObjectMember>"#,
        );
        assert_eq!(xml, expected);
    }
}
