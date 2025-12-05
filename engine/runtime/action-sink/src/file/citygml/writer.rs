use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use super::converter::{
    format_pos_list, BoundingEnvelope, CityObjectType, GeometryEntry, GmlElement, GmlSurface,
};
use crate::errors::SinkError;

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
        }
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

        for entry in geometries {
            self.write_lod_geometry(city_type, entry)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(element_name)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_lod_geometry(
        &mut self,
        city_type: CityObjectType,
        entry: &GeometryEntry,
    ) -> Result<(), SinkError> {
        let ns = city_type.namespace_prefix();
        let lod_elem = self.get_geometry_element_name(ns, entry, city_type);

        match &entry.element {
            GmlElement::Solid { id, surfaces } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_solid(id.as_deref(), surfaces)?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            GmlElement::MultiSurface { id, surfaces } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_multi_surface(id.as_deref(), surfaces)?;

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

    fn write_solid(&mut self, id: Option<&str>, surfaces: &[GmlSurface]) -> Result<(), SinkError> {
        let mut solid = BytesStart::new("gml:Solid");
        if let Some(gml_id) = id {
            solid.push_attribute(("gml:id", gml_id));
        }
        // Add srsName to geometry for proper CRS reference
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
            self.write_surface_member(surface)?;
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
    ) -> Result<(), SinkError> {
        let mut ms = BytesStart::new("gml:MultiSurface");
        if let Some(gml_id) = id {
            ms.push_attribute(("gml:id", gml_id));
        }
        // Add srsName to geometry for proper CRS reference
        ms.push_attribute(("srsName", self.srs_name.as_str()));
        ms.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(ms))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for surface in surfaces {
            self.write_surface_member(surface)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:MultiSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_surface_member(&mut self, surface: &GmlSurface) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:surfaceMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut polygon = BytesStart::new("gml:Polygon");
        if let Some(id) = &surface.id {
            polygon.push_attribute(("gml:id", id.as_str()));
        }
        self.writer
            .write_event(Event::Start(polygon))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Exterior ring
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.write_linear_ring(&surface.exterior)?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Interior rings (holes)
        for interior in &surface.interiors {
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:interior")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.write_linear_ring(interior)?;
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

        Ok(())
    }

    fn write_linear_ring(
        &mut self,
        coords: &[reearth_flow_geometry::types::coordinate::Coordinate3D<f64>],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:LinearRing")))
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
        self.writer
            .write_event(Event::End(BytesEnd::new("core:CityModel")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }
}
