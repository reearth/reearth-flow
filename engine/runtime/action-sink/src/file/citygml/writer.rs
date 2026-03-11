use std::collections::HashMap;
use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use reearth_flow_types::material::{Texture, X3DMaterial};

use super::converter::{
    format_pos_list, AppearanceBundle, BoundingEnvelope, CityObjectType, GeometryEntry, GmlElement,
    GmlSurface,
};
use crate::errors::SinkError;

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

pub struct CityGmlXmlWriter<W: Write> {
    writer: Writer<W>,
    srs_name: String,
    id_counter: u64,
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
            id_counter: 0,
            pending_appearances: Vec::new(),
            uri_remap: HashMap::new(),
        }
    }

    pub fn set_uri_remap(&mut self, remap: HashMap<String, String>) {
        self.uri_remap = remap;
    }

    fn generate_gml_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}_{}", prefix, self.id_counter)
    }

    pub fn write_header(&mut self, envelope: Option<&BoundingEnvelope>) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut city_model = BytesStart::new("core:CityModel");
        for (prefix, uri) in CITYGML_2_NAMESPACES {
            city_model.push_attribute((*prefix, *uri));
        }
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

        self.write_text_element("gml:lowerCorner", &envelope.lower_corner_str())?;
        self.write_text_element("gml:upperCorner", &envelope.upper_corner_str())?;

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
        geometries: &[GeometryEntry],
        gml_id: Option<&str>,
        appearance: Option<&AppearanceBundle>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let element_name = city_type.element_name();
        let mut city_obj_elem = BytesStart::new(element_name);
        let obj_id = gml_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.generate_gml_id(city_type.id_prefix()));
        city_obj_elem.push_attribute(("gml:id", obj_id.as_str()));
        self.writer
            .write_event(Event::Start(city_obj_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let need_appearance = appearance.is_some_and(|a| a.has_content());
        let mut surface_appearances: Vec<SurfaceAppearance> = Vec::new();

        for entry in geometries {
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

        match &entry.element {
            GmlElement::Solid { id, surfaces } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_solid(
                    id.as_deref(),
                    surfaces,
                    need_appearance,
                    surface_appearances,
                )?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            GmlElement::MultiSurface { id, surfaces } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_multi_surface(
                    id.as_deref(),
                    surfaces,
                    need_appearance,
                    surface_appearances,
                )?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            GmlElement::MultiCurve { id, curves } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_multi_curve(id.as_deref(), curves)?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn get_geometry_element_name(
        &self,
        ns: &str,
        entry: &GeometryEntry,
        city_type: CityObjectType,
    ) -> String {
        if let Some(property) = &entry.property {
            format!("{}:{}", ns, property)
        } else {
            // GenericCityObject uses lodXGeometry, not lodXMultiSurface/lodXSolid
            if city_type == CityObjectType::GenericCityObject {
                format!("{}:lod{}Geometry", ns, entry.lod)
            } else {
                let geom_type = match &entry.element {
                    GmlElement::Solid { .. } => "Solid",
                    GmlElement::MultiSurface { .. } => "MultiSurface",
                    GmlElement::MultiCurve { .. } => "MultiCurve",
                };
                format!("{}:lod{}{}", ns, entry.lod, geom_type)
            }
        }
    }

    fn write_solid(
        &mut self,
        id: Option<&str>,
        surfaces: &[GmlSurface],
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        let mut solid = BytesStart::new("gml:Solid");
        if let Some(gml_id) = id {
            solid.push_attribute(("gml:id", gml_id));
        }
        solid.push_attribute(("srsName", self.srs_name.as_str()));
        solid.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(solid))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("gml:exterior")))
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
            .write_event(Event::End(BytesEnd::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Solid")))
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
            Some(self.generate_gml_id("poly"))
        } else {
            surface.id.clone()
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
        coords: &[reearth_flow_geometry::types::coordinate::Coordinate3D<f64>],
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
        curves: &[Vec<reearth_flow_geometry::types::coordinate::Coordinate3D<f64>>],
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

        self.write_text_element("app:theme", "rgbTexture")?;

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

        let c = &material.diffuse_color;
        self.write_text_element("app:diffuseColor", &format!("{} {} {}", c.r, c.g, c.b))?;
        let c = &material.specular_color;
        self.write_text_element("app:specularColor", &format!("{} {} {}", c.r, c.g, c.b))?;
        self.write_text_element(
            "app:ambientIntensity",
            &material.ambient_intensity.to_string(),
        )?;
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
        texture: &Texture,
        targets: &[&SurfaceAppearance],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Start(BytesStart::new("app:ParameterizedTexture")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let image_uri = self
            .uri_remap
            .get(texture.uri.as_str())
            .cloned()
            .unwrap_or_else(|| texture.uri.to_string());
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

    pub fn write_footer(&mut self) -> Result<(), SinkError> {
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
        self.writer
            .write_event(Event::End(BytesEnd::new("core:CityModel")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }
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
    use reearth_flow_geometry::types::coordinate::Coordinate3D;
    use reearth_flow_types::material::{Texture, X3DMaterial};
    use url::Url;

    use super::*;

    const SRS: &str = "http://www.opengis.net/def/crs/EPSG/0/6697";

    // Exterior: 3 coords → posList "35 139 0 35 139.1 0 35.1 139 0"
    // (format_pos_list emits y x z, integer-valued floats drop the decimal)
    fn triangle() -> Vec<Coordinate3D<f64>> {
        vec![
            Coordinate3D::new__(139.0, 35.0, 0.0),
            Coordinate3D::new__(139.1, 35.0, 0.0),
            Coordinate3D::new__(139.0, 35.1, 0.0),
        ]
    }

    /// Run write_city_object + write_footer on a single LOD-2 MultiSurface for a Building
    /// and return the compact XML string (appearances emitted at CityModel level).
    fn write_building(surfaces: Vec<GmlSurface>, appearance: Option<&AppearanceBundle>) -> String {
        let entry = GeometryEntry {
            lod: 2,
            property: None,
            element: GmlElement::MultiSurface { id: None, surfaces },
        };
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.write_city_object(
            CityObjectType::Building,
            &[entry],
            Some("obj-001"),
            appearance,
        )
        .unwrap();
        w.write_footer().unwrap();
        String::from_utf8(buf).unwrap()
    }

    // ── X3DMaterial ────────────────────────────────────────────────────────────

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
            r#"<app:diffuseColor>0.7 0.7 0.7</app:diffuseColor>"#,
            r#"<app:specularColor>0.04 0.04 0.04</app:specularColor>"#,
            r#"<app:ambientIntensity>0.9</app:ambientIntensity>"#,
            r#"<app:target>#poly_1</app:target>"#,
            r#"</app:X3DMaterial></app:surfaceDataMember>"#,
            r#"</app:Appearance>"#,
            r#"</app:appearanceMember>"#,
            r#"</core:CityModel>"#,
        );
        assert_eq!(xml, expected);
    }

    // ── ParameterizedTexture ───────────────────────────────────────────────────

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
            materials: vec![],
            textures: vec![Texture {
                uri: Url::parse("file:///textures/wall.jpg").unwrap(),
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
            r#"</core:CityModel>"#,
        );
        assert_eq!(xml, expected);
    }
}
