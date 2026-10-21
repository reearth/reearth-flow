use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use reearth_flow_citygml::schema::{ClassModel, QName, SchemaSet};
use reearth_flow_types::material::X3DMaterial;

// The seam is geometry-neutral and shared; only the `posList` formatter is
// world-specific, and `converter` resolves to the compiled world's module, so
// this file needs no `cfg` to pick the right one.
use super::classes::{CityClass, Prefixes};
use super::converter::format_pos_list;
use super::model::{
    AppearanceBundle, BoundingEnvelope, GeometryEntry, GmlElement, GmlSolid, GmlSurface,
    GmlTexture, ObjectNode,
};
use super::placement::Placed;
use super::properties::XmlProperty;
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

/// What writing one object tree did that the caller reports.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WriteOutcome {
    /// `lod1MultiSurface` properties dropped by the LOD1 exclusive-shell rule,
    /// per [`ObjectNode::source`]. A source with none has no entry.
    pub lod1_surfaces_dropped: BTreeMap<usize, usize>,
}

/// The state one [`CityGmlXmlWriter::write_object_tree`] call threads through
/// the objects it writes.
struct TreeWrite<'s, 'a> {
    schema: &'s SchemaSet,
    /// Indexed by [`ObjectNode::source`].
    appearances: &'a [Option<&'a AppearanceBundle>],
    /// The surfaces each source's appearance targets, by source in the order
    /// the sources were first met.
    surfaces: Vec<(usize, Vec<SurfaceAppearance>)>,
    outcome: WriteOutcome,
}

impl TreeWrite<'_, '_> {
    /// The index in `surfaces` of `source`'s list, added if it is new.
    fn surfaces_of(&mut self, source: usize) -> usize {
        match self.surfaces.iter().position(|(seen, _)| *seen == source) {
            Some(index) => index,
            None => {
                self.surfaces.push((source, Vec::new()));
                self.surfaces.len() - 1
            }
        }
    }
}

/// Shared with `classes`, which resolves feature types against these prefixes.
pub(super) const CITYGML_2_NAMESPACES: &[(&str, &str)] = &[
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

pub struct CityGmlXmlWriter<'a, W: Write> {
    writer: Writer<W>,
    srs_name: String,
    /// The schemas classes are written against; `None` is the core CityGML 2.0
    /// set.
    schema: Option<&'a SchemaSet>,
    /// Names the namespace each geometry property is declared in.
    prefixes: Prefixes,
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

impl<'a, W: Write> CityGmlXmlWriter<'a, W> {
    pub fn new(inner: W, pretty: bool, srs_name: String) -> Self {
        let writer = if pretty {
            Writer::new_with_indent(inner, b' ', 2)
        } else {
            Writer::new(inner)
        };
        Self {
            writer,
            srs_name,
            schema: None,
            prefixes: Prefixes::citygml2(),
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

    /// Write classes against `schema`, a set with ADEs loaded, whose
    /// namespaces `prefixes` binds.
    pub fn set_schema(&mut self, schema: &'a SchemaSet, prefixes: Prefixes) {
        self.schema = Some(schema);
        self.prefixes = prefixes;
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

    /// Declares every ADE prefix `set_schema` bound, the same table classes
    /// are written with. `ade_locations` pairs each ADE namespace with the
    /// location the output declares for it.
    pub fn write_header(
        &mut self,
        envelope: Option<&BoundingEnvelope>,
        ade_locations: &[(String, String)],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut city_model = BytesStart::new("core:CityModel");
        for (prefix, uri) in CITYGML_2_NAMESPACES {
            city_model.push_attribute((*prefix, *uri));
        }
        for (prefix, uri) in self.prefixes.ade() {
            city_model.push_attribute((format!("xmlns:{prefix}").as_str(), uri));
        }
        let mut schema_location = CITYGML_2_SCHEMA_LOCATION.to_string();
        for (namespace, location) in ade_locations {
            schema_location.push_str(&format!(" {namespace} {location}"));
        }
        city_model.push_attribute(("xsi:schemaLocation", schema_location.as_str()));
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

    /// Write one top-level city object with no objects nested in it.
    pub fn write_city_object(
        &mut self,
        class: &CityClass,
        geometries: Vec<GeometryEntry>,
        properties: Vec<Placed>,
        gml_id: Option<&str>,
        appearance: Option<&AppearanceBundle>,
    ) -> Result<(), SinkError> {
        let object = ObjectNode {
            class: class.clone(),
            key: None,
            synthetic: false,
            gml_id: gml_id.map(str::to_owned),
            geometries,
            properties,
            children: Vec::new(),
            source: 0,
        };
        self.write_object_tree(object, &[appearance])?;
        Ok(())
    }

    /// Write `object` as one `core:cityObjectMember`, each nested object
    /// inside its parent's property.
    ///
    /// `appearances` is indexed by [`ObjectNode::source`]: an object's
    /// surfaces resolve against its own source's palettes, and each source
    /// gets its own `app:appearanceMember`, in the order its objects are
    /// first met.
    pub fn write_object_tree(
        &mut self,
        object: ObjectNode,
        appearances: &[Option<&AppearanceBundle>],
    ) -> Result<WriteOutcome, SinkError> {
        let schema = match self.schema {
            Some(schema) => schema,
            None => SchemaSet::core().map_err(|e| {
                SinkError::CityGmlWriter(format!("the CityGML 2.0 schemas failed to load: {e}"))
            })?,
        };

        self.writer
            .write_event(Event::Start(BytesStart::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut tree = TreeWrite {
            schema,
            appearances,
            surfaces: Vec::new(),
            outcome: WriteOutcome::default(),
        };
        self.write_object(object, &mut tree)?;

        self.writer
            .write_event(Event::End(BytesEnd::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Appearance is written at CityModel level, as `app:appearanceMember`.
        for (source, surfaces) in tree.surfaces {
            if surfaces.is_empty() {
                continue;
            }
            if let Some(appearance) = appearances.get(source).copied().flatten() {
                self.pending_appearances
                    .push((appearance.clone(), surfaces));
            }
        }

        Ok(tree.outcome)
    }

    /// Write one city object element and everything nested in it.
    fn write_object(
        &mut self,
        object: ObjectNode,
        tree: &mut TreeWrite<'_, '_>,
    ) -> Result<(), SinkError> {
        let ObjectNode {
            class,
            gml_id,
            geometries,
            properties,
            children: nested,
            source,
            ..
        } = object;

        let element_name = class.name();
        let mut city_obj_elem = BytesStart::new(element_name.as_str());
        let obj_id = self.claim_gml_id(gml_id.as_deref(), &class.id_prefix);
        city_obj_elem.push_attribute(("gml:id", obj_id.as_str()));
        self.writer
            .write_event(Event::Start(city_obj_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // The caller passes one entry per source, so a missing one is a bug,
        // not an object without appearance.
        debug_assert!(
            source < tree.appearances.len(),
            "no appearance entry for source {source}"
        );
        // Registered before anything inside is written, so a parent's source
        // comes before its children's.
        let surfaces_at = tree.surfaces_of(source);
        let need_appearance = tree
            .appearances
            .get(source)
            .copied()
            .flatten()
            .is_some_and(|a| a.has_content());

        let (merged, dropped) = settle_geometry(geometries, &class, tree.schema);
        if dropped > 0 {
            self.dropped_lod1_surfaces += dropped;
            *tree
                .outcome
                .lod1_surfaces_dropped
                .entry(source)
                .or_insert(0) += dropped;
        }

        // CityGML declares each class's properties as an `xs:sequence`, so
        // arrival order is not good enough: geometry, attribute properties and
        // nested objects share one list and are emitted in schema order. The
        // class model is the same for every entry, so look it up once; the key
        // is computed once per entry and the sort stays stable. Placement and
        // containment numbered the other slots against this same model.
        let model = tree.schema.class_for_element(&class.element);
        enum Child<'a> {
            /// The wrapper's qualified name, `prefix:local`, and its geometry.
            Geometry(String, &'a GeometryEntry),
            Property(XmlProperty),
            /// The wrapper's qualified name and the object it holds.
            Nested(String, ObjectNode),
        }
        // Geometry, then properties, then nested objects, so equal slots keep
        // that order under the stable sort.
        let mut children: Vec<(usize, Child)> = merged
            .iter()
            .map(|entry| {
                let local = geometry_property_name(entry, &class);
                let property = property_qname(model, &class, &self.prefixes, local);
                let slot = slot_position(model, &property);
                (
                    slot,
                    Child::Geometry(self.qualified(&property, &class), entry),
                )
            })
            .chain(
                properties
                    .into_iter()
                    .map(|placed| (placed.slot, Child::Property(placed.property))),
            )
            .chain(nested.into_iter().map(|nested| {
                (
                    nested.slot,
                    Child::Nested(self.qualified(&nested.property, &class), nested.object),
                )
            }))
            .collect();
        children.sort_by_key(|(slot, _)| *slot);

        for (_, child) in children {
            match child {
                Child::Geometry(property, entry) => self.write_lod_geometry(
                    &property,
                    entry,
                    need_appearance,
                    &mut tree.surfaces[surfaces_at].1,
                )?,
                Child::Property(property) => self.write_property(&property, false)?,
                Child::Nested(property, object) => {
                    self.writer
                        .write_event(Event::Start(BytesStart::new(property.as_str())))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                    self.write_object(object, tree)?;
                    self.writer
                        .write_event(Event::End(BytesEnd::new(property.as_str())))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                }
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(element_name.as_str())))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    /// `property` as an element name in an object of `class`: `prefix:local`,
    /// in the class's own prefix when no prefix is bound to its namespace.
    fn qualified(&self, property: &QName, class: &CityClass) -> String {
        let prefix = self
            .prefixes
            .prefix_of(&property.namespace)
            .unwrap_or(&class.prefix);
        format!("{prefix}:{}", property.local)
    }

    /// Write one attribute-built element and its descendants. `in_xal` is
    /// whether an ancestor already declared `xmlns:xAL`. A nested `gml:id` is
    /// claimed like every other, so it stays unique in the document.
    ///
    /// `xAL` is the one prefix in the table not declared on the root, so it is
    /// declared on the outermost element that uses it instead, and documents
    /// without such content keep exactly the root they always had.
    fn write_property(&mut self, property: &XmlProperty, in_xal: bool) -> Result<(), SinkError> {
        let name = format!("{}:{}", property.prefix, property.local);
        let mut start = BytesStart::new(name.as_str());
        let declares_xal = property.prefix == "xAL" && !in_xal;
        if declares_xal {
            if let Some(uri) = self.prefixes.namespace("xAL") {
                start.push_attribute(("xmlns:xAL", uri));
            }
        }
        for (key, value) in &property.attrs {
            if key == "gml:id" {
                let id = self.claim_gml_id(Some(value), &property.local);
                start.push_attribute(("gml:id", id.as_str()));
            } else {
                start.push_attribute((key.as_str(), value.as_str()));
            }
        }
        if property.text.is_none() && property.children.is_empty() {
            return self
                .writer
                .write_event(Event::Empty(start))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()));
        }
        self.writer
            .write_event(Event::Start(start))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        if let Some(text) = &property.text {
            self.writer
                .write_event(Event::Text(BytesText::new(text)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }
        for child in &property.children {
            self.write_property(child, in_xal || declares_xal)?;
        }
        self.writer
            .write_event(Event::End(BytesEnd::new(name.as_str())))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }

    /// `property` is the wrapper's qualified name, `prefix:local`.
    fn write_lod_geometry(
        &mut self,
        property: &str,
        entry: &GeometryEntry,
        need_appearance: bool,
        surface_appearances: &mut Vec<SurfaceAppearance>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new(property)))
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
            .write_event(Event::End(BytesEnd::new(property)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
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
fn geometry_property_name(entry: &GeometryEntry, class: &CityClass) -> String {
    if let Some(property) = &entry.property {
        return property.clone();
    }
    if class.is_generic() {
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

/// The geometry steps every object runs, top-level or nested, against its
/// own class. Returns the entries to write and how many `lod1MultiSurface`
/// properties the LOD1 exclusive-shell rule dropped.
///
/// Schema order is the `xs:sequence` half of the content model; cardinality is
/// the other half, so duplicate geometry collapses here, before the caller
/// sorts, rather than being written as siblings.
fn settle_geometry(
    entries: Vec<GeometryEntry>,
    class: &CityClass,
    schema: &SchemaSet,
) -> (Vec<GeometryEntry>, usize) {
    let mut merged = merge_duplicate_properties(entries, class);
    let dropped = enforce_lod1_shell(&mut merged, class, schema);
    (merged, dropped)
}

/// Collapse entries that resolve to the same property name into one element.
///
/// Every `lodNXxx` property is declared `minOccurs="0"` with no `maxOccurs` in
/// the CityGML schemas, so it may appear at most once; `boundedBy` is the only
/// repeating one. One object can still arrive with two entries naming the same
/// property: several geometry members of one owner, a non-city-object owner
/// folded into its parent, or, in the legacy world, nested objects flattened
/// onto the feature. Writing both produces a document no validator accepts, so
/// they merge into a single multi-geometry here.
///
/// This keeps the geometry while losing which member each piece came from.
/// Each piece belongs to the same object either way, so nothing survives the
/// merge that would have survived without it.
fn merge_duplicate_properties(
    entries: Vec<GeometryEntry>,
    class: &CityClass,
) -> Vec<GeometryEntry> {
    let mut merged: Vec<GeometryEntry> = Vec::with_capacity(entries.len());
    for entry in entries {
        let name = geometry_property_name(&entry, class);
        let unfolded = match merged
            .iter_mut()
            .find(|held| geometry_property_name(held, class) == name)
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

/// The heads whose substitutes carry the LOD1 exclusive-shell rule.
const EXCLUSIVE_LOD1_SHELL_HEADS: [(&str, &str); 3] = [
    (
        "http://www.opengis.net/citygml/building/2.0",
        "_AbstractBuilding",
    ),
    (
        "http://www.opengis.net/citygml/bridge/2.0",
        "_AbstractBridge",
    ),
    (
        "http://www.opengis.net/citygml/tunnel/2.0",
        "_AbstractTunnel",
    ),
];

/// Whether this class inherits the LOD1 exclusive-shell rule.
///
/// CityGML 2.0 states it identically for `_AbstractBuilding` (§10.3.4),
/// `_AbstractTunnel` and `_AbstractBridge`: at LOD1 the volumetric and surface
/// parts of the exterior shell are identical, so either `lodNSolid` or
/// `lodNMultiSurface` must be used, but not both, and from LOD2 the two may be
/// modelled individually and complementary. `WaterBody` declares the same pair
/// at LOD1 and carries no such restriction, so it is deliberately absent here.
/// An ADE class substituting for one of the three inherits the rule with it.
fn has_exclusive_lod1_shell(class: &CityClass, schema: &SchemaSet) -> bool {
    EXCLUSIVE_LOD1_SHELL_HEADS.iter().any(|(namespace, local)| {
        schema.substitutes_for(&class.element, &QName::new(*namespace, *local))
    })
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
fn enforce_lod1_shell(
    entries: &mut Vec<GeometryEntry>,
    class: &CityClass,
    schema: &SchemaSet,
) -> usize {
    if !has_exclusive_lod1_shell(class, schema) {
        return 0;
    }
    let names: Vec<String> = entries
        .iter()
        .map(|entry| geometry_property_name(entry, class))
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
                owner: extra.owner,
                element,
            })
        }
    }
    None
}

/// Where the property `local` sits in `class`'s content model, or
/// `usize::MAX` for a name no slot accepts. A property the schema does not
/// declare sorts last rather than being dropped, so invalid input stays visible
/// to the schema gate instead of vanishing, and it keeps its arrival order
/// under the stable sort.
#[cfg(test)]
fn property_position(schema: &SchemaSet, class: &CityClass, local: &str) -> usize {
    let model = schema.class_for_element(&class.element);
    let property = property_qname(model, class, &Prefixes::citygml2(), local.to_string());
    slot_position(model, &property)
}

fn slot_position(model: Option<&ClassModel>, property: &QName) -> usize {
    model
        .and_then(|model| model.slot_index(property))
        .unwrap_or(usize::MAX)
}

/// The property `local` of `class`, in the namespace of the first slot that
/// declares that local name: an ADE class writes the `bldg:lod2Solid` it
/// inherits, not `uro:lod2Solid`. A name no slot declares, or one declared in a
/// namespace `prefixes` cannot write, stays in the class's own namespace.
fn property_qname(
    model: Option<&ClassModel>,
    class: &CityClass,
    prefixes: &Prefixes,
    local: String,
) -> QName {
    let namespace = model
        .into_iter()
        .flat_map(|model| model.slots())
        .filter_map(|slot| slot.declared.as_ref())
        .find(|declared| declared.local == local)
        .map(|declared| declared.namespace.as_str())
        .filter(|namespace| prefixes.prefix_of(namespace).is_some())
        .unwrap_or(class.element.namespace.as_str());
    QName::new(namespace, local)
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
    use crate::file::citygml::model::NestedObject;

    const SRS: &str = "http://www.opengis.net/def/crs/EPSG/0/6697";

    use reearth_flow_citygml::schema::SchemaSet;

    use super::super::classes;

    const URO: &str = "https://www.geospatial.jp/iur/uro/3.1";

    /// The 17 classes the writer wrote before classes resolved by name.
    const ALL_CITY_TYPES: [&str; 17] = [
        "bldg:Building",
        "bldg:BuildingPart",
        "tran:Road",
        "tran:Railway",
        "tran:Track",
        "tran:Square",
        "brid:Bridge",
        "brid:BridgePart",
        "tun:Tunnel",
        "tun:TunnelPart",
        "wtr:WaterBody",
        "luse:LandUse",
        "veg:SolitaryVegetationObject",
        "veg:PlantCover",
        "frn:CityFurniture",
        "dem:ReliefFeature",
        "gen:GenericCityObject",
    ];

    /// A core class, resolved as the sink resolves it.
    fn class(value: &str) -> CityClass {
        classes::resolve(value, &Prefixes::citygml2(), SchemaSet::core().unwrap()).unwrap()
    }

    /// The i-UR 3.1 schema from the PLATEAU fixture, with `uro` bound.
    fn iur() -> (SchemaSet, Prefixes) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testing/data/fixtures/plateau-citymodel/15202_nagaoka-shi_city_2024_citygml_1_op/schemas/iur/uro/3.1/urbanObject.xsd");
        let set = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes = Prefixes::with_ade(&[("uro".into(), URO.into())]);
        (set, prefixes)
    }

    /// Every name `geometry_property_name` can produce for a class.
    fn geometry_names(city_type: &CityClass) -> Vec<String> {
        let families: &[&str] = if city_type.is_generic() {
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
        for value in ALL_CITY_TYPES {
            assert!(
                schema.class_for_element(&class(value).element).is_some(),
                "{value}"
            );
        }
    }

    /// The switch must not reorder anything: a name the old table listed keeps
    /// its relative order, and a name it did not list still sorts last.
    #[test]
    fn geometry_properties_order_exactly_as_the_generated_table_did() {
        let schema = SchemaSet::core().unwrap();
        let frozen = frozen_order();
        for value in ALL_CITY_TYPES {
            let city_type = class(value);
            let old = &frozen[&city_type.element.local];
            let mut listed: Vec<(usize, usize)> = Vec::new();
            // The names the writer builds, plus every `lod*` property the old
            // table listed for the class: converters supply some of those
            // (`lodNGeometry`, terrain intersections, `lod0FootPrint`, ...).
            let mut names = geometry_names(&city_type);
            names.extend(old.iter().filter(|p| p.starts_with("lod")).cloned());
            names.sort();
            names.dedup();
            for name in names {
                let new = property_position(schema, &city_type, &name);
                match old.iter().position(|p| *p == name) {
                    Some(old_index) => {
                        assert_ne!(new, usize::MAX, "{value} {name} lost its slot");
                        listed.push((old_index, new));
                    }
                    None => assert_eq!(new, usize::MAX, "{value} {name} gained a slot"),
                }
            }
            listed.sort();
            assert!(
                listed.windows(2).all(|w| w[0].1 < w[1].1),
                "{value} order changed: {listed:?}"
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
        w.write_city_object(
            &class("bldg:Building"),
            entries,
            Vec::new(),
            Some("obj-001"),
            None,
        )
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
                owner: Default::default(),
                element: GmlElement::MultiSurface {
                    id: None,
                    surfaces: vec![plain_surface(triangle())],
                },
            },
            GeometryEntry {
                lod: 2,
                property: Some("lod2MultiSurface".to_string()),
                owner: Default::default(),
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
                owner: Default::default(),
                element: GmlElement::MultiCurve {
                    id: None,
                    curves: vec![triangle()],
                },
            },
            GeometryEntry {
                lod: 0,
                property: Some("lod0RoofEdge".to_string()),
                owner: Default::default(),
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
                owner: Default::default(),
                element: GmlElement::Solid(solid(triangle())),
            },
            GeometryEntry {
                lod: 2,
                property: Some("lod2Solid".to_string()),
                owner: Default::default(),
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
                owner: Default::default(),
                element: GmlElement::Solid(solid_of(triangle())),
            },
            GeometryEntry {
                lod,
                property: Some(format!("lod{lod}MultiSurface")),
                owner: Default::default(),
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

    /// Write `entries` as one `class` object with a minted id, through a writer
    /// given `schema` and `prefixes`. Returns the XML and the LOD1 drop count.
    fn write_with(
        schema: &SchemaSet,
        prefixes: &Prefixes,
        class: &CityClass,
        entries: Vec<GeometryEntry>,
    ) -> (String, usize) {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.set_schema(schema, prefixes.clone());
        w.write_city_object(class, entries, Vec::new(), None, None)
            .unwrap();
        let dropped = w.dropped_lod1_surfaces();
        (String::from_utf8(buf).unwrap(), dropped)
    }

    /// The root start tag, without the XML declaration.
    fn root(prefixes: &Prefixes, ade_locations: &[(String, String)]) -> String {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.set_schema(SchemaSet::core().unwrap(), prefixes.clone());
        w.write_header(None, ade_locations).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        xml.split_once("?>").unwrap().1.to_string()
    }

    #[test]
    fn an_ade_class_writes_its_inherited_geometry_in_the_declaring_namespace() {
        // uro:UndergroundBuilding extends bldg:AbstractBuildingType, so lod2Solid belongs to bldg.
        let (set, prefixes) = iur();
        let class = classes::resolve("uro:UndergroundBuilding", &prefixes, &set).unwrap();
        let (xml, _) = write_with(&set, &prefixes, &class, shell_entries(2));
        assert!(
            xml.contains("<uro:UndergroundBuilding gml:id=\"undergroundbuilding_1\">"),
            "{xml}"
        );
        assert!(xml.contains("<bldg:lod2Solid>"), "{xml}");
        assert!(!xml.contains("uro:lod2Solid"), "{xml}");
    }

    #[test]
    fn the_lod1_shell_rule_follows_the_substitution_chain() {
        // shell_entries(1) carries lod1Solid and lod1MultiSurface; an ADE building keeps only the solid.
        let (set, prefixes) = iur();
        let class = classes::resolve("uro:UndergroundBuilding", &prefixes, &set).unwrap();
        let (xml, dropped_lod1_surfaces) = write_with(&set, &prefixes, &class, shell_entries(1));
        assert_eq!(dropped_lod1_surfaces, 1, "{xml}");
    }

    #[test]
    fn the_root_declares_each_ade_namespace_and_its_declared_location() {
        let prefixes = Prefixes::with_ade(&[("uro".into(), URO.into())]);
        let tag = root(
            &prefixes,
            &[(
                URO.to_string(),
                "../../schemas/iur/uro/3.1/urbanObject.xsd".to_string(),
            )],
        );
        assert!(
            tag.contains(r#"xmlns:uro="https://www.geospatial.jp/iur/uro/3.1""#),
            "{tag}"
        );
        let location = tag
            .split_once("xsi:schemaLocation=\"")
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0;
        assert!(
            location.ends_with(
                "https://www.geospatial.jp/iur/uro/3.1 ../../schemas/iur/uro/3.1/urbanObject.xsd"
            ),
            "{location}"
        );
    }

    #[test]
    fn with_no_ade_the_root_is_unchanged() {
        // Captured from the writer's output before classes resolved by name.
        let expected = concat!(
            r#"<core:CityModel"#,
            r#" xmlns:core="http://www.opengis.net/citygml/2.0""#,
            r#" xmlns:gml="http://www.opengis.net/gml""#,
            r#" xmlns:bldg="http://www.opengis.net/citygml/building/2.0""#,
            r#" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0""#,
            r#" xmlns:brid="http://www.opengis.net/citygml/bridge/2.0""#,
            r#" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0""#,
            r#" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0""#,
            r#" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0""#,
            r#" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0""#,
            r#" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0""#,
            r#" xmlns:dem="http://www.opengis.net/citygml/relief/2.0""#,
            r#" xmlns:gen="http://www.opengis.net/citygml/generics/2.0""#,
            r#" xmlns:app="http://www.opengis.net/citygml/appearance/2.0""#,
            r#" xmlns:xlink="http://www.w3.org/1999/xlink""#,
            r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#,
            r#" xsi:schemaLocation=""#,
            r#"http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd"#,
            r#" http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd"#,
            r#" http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd"#,
            r#" http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd"#,
            r#" http://www.opengis.net/citygml/bridge/2.0 http://schemas.opengis.net/citygml/bridge/2.0/bridge.xsd"#,
            r#" http://www.opengis.net/citygml/tunnel/2.0 http://schemas.opengis.net/citygml/tunnel/2.0/tunnel.xsd"#,
            r#" http://www.opengis.net/citygml/waterbody/2.0 http://schemas.opengis.net/citygml/waterbody/2.0/waterBody.xsd"#,
            r#" http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd"#,
            r#" http://www.opengis.net/citygml/vegetation/2.0 http://schemas.opengis.net/citygml/vegetation/2.0/vegetation.xsd"#,
            r#" http://www.opengis.net/citygml/cityfurniture/2.0 http://schemas.opengis.net/citygml/cityfurniture/2.0/cityFurniture.xsd"#,
            r#" http://www.opengis.net/citygml/relief/2.0 http://schemas.opengis.net/citygml/relief/2.0/relief.xsd"#,
            r#" http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd"#,
            r#" http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd"#,
            r#"">"#,
        );
        assert_eq!(root(&Prefixes::citygml2(), &[]), expected);
    }

    /// Write one geometry entry as a Building city object and return the XML.
    fn write_entry(entry: GeometryEntry, appearance: Option<&AppearanceBundle>) -> String {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        w.write_city_object(
            &class("bldg:Building"),
            vec![entry],
            Vec::new(),
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
                owner: Default::default(),
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
            &class("bldg:Building"),
            vec![GeometryEntry {
                lod: 2,
                property: None,
                owner: Default::default(),
                element: GmlElement::MultiSurface {
                    id: None,
                    surfaces: vec![surface],
                },
            }],
            Vec::new(),
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
            owner: Default::default(),
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
            owner: Default::default(),
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
            owner: Default::default(),
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

    fn text_prop(prefix: &str, local: &str, text: &str) -> XmlProperty {
        let mut p = XmlProperty::new(prefix, local);
        p.text = Some(text.to_owned());
        p
    }

    fn write_with_properties(entries: Vec<GeometryEntry>, properties: Vec<Placed>) -> String {
        let mut buf = Vec::new();
        {
            let mut writer = CityGmlXmlWriter::new(&mut buf, false, "EPSG:6697".to_string());
            writer
                .write_city_object(
                    &class("bldg:Building"),
                    entries,
                    properties,
                    Some("b1"),
                    None,
                )
                .unwrap();
        }
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn properties_and_geometry_share_one_schema_order() {
        let schema = SchemaSet::core().unwrap();
        let (placed, _) = crate::file::citygml::placement::place(
            schema,
            &Prefixes::citygml2(),
            &class("bldg:Building").element,
            vec![
                text_prop("bldg", "measuredHeight", "10.5"),
                text_prop("gen", "stringAttribute", "x"),
            ],
        );
        let xml = write_with_properties(shell_entries(1), placed);
        let gen = xml.find("<gen:stringAttribute").unwrap();
        let height = xml.find("<bldg:measuredHeight").unwrap();
        let solid = xml.find("<bldg:lod1Solid").unwrap();
        assert!(gen < height && height < solid, "{xml}");
    }

    #[test]
    fn attributes_text_and_nested_ids_are_written() {
        let mut value = text_prop("gen", "value", "1.5");
        value.attrs.push(("uom".to_owned(), "kWh".to_owned()));
        let mut measure = XmlProperty::new("gen", "measureAttribute");
        measure.attrs.push(("name".to_owned(), "日射量".to_owned()));
        measure.children.push(value);
        let mut address = XmlProperty::new("core", "Address");
        address.attrs.push(("gml:id".to_owned(), "b1".to_owned()));
        let placed = vec![
            Placed {
                slot: 0,
                property: measure,
            },
            Placed {
                slot: 1,
                property: address,
            },
        ];
        let xml = write_with_properties(Vec::new(), placed);
        assert!(
            xml.contains(
                r#"<gen:measureAttribute name="日射量"><gen:value uom="kWh">1.5</gen:value></gen:measureAttribute>"#
            ),
            "{xml}"
        );
        // `b1` is the object's own id, so the nested one is minted afresh.
        assert!(
            xml.contains(r#"<core:Address gml:id="Address_1"/>"#),
            "{xml}"
        );
    }

    #[test]
    fn xal_is_declared_on_its_outermost_element_only() {
        let mut details = XmlProperty::new("xAL", "AddressDetails");
        details
            .children
            .push(text_prop("xAL", "CountryName", "日本"));
        let mut xal = XmlProperty::new("core", "xalAddress");
        xal.children.push(details);
        let xml = write_with_properties(
            Vec::new(),
            vec![Placed {
                slot: 0,
                property: xal,
            }],
        );
        assert_eq!(xml.matches("xmlns:xAL=").count(), 1, "{xml}");
        assert!(
            xml.contains(
                r#"<xAL:AddressDetails xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0">"#
            ),
            "{xml}"
        );
    }

    // Object trees

    /// A childless object of the core class `value`, from source 0.
    fn node(value: &str, id: Option<&str>, entries: Vec<GeometryEntry>) -> ObjectNode {
        ObjectNode {
            class: class(value),
            key: id.map(str::to_owned),
            synthetic: false,
            gml_id: id.map(str::to_owned),
            geometries: entries,
            properties: Vec::new(),
            children: Vec::new(),
            source: 0,
        }
    }

    /// Nest `child` in `parent` at the slot the core schema gives it, which
    /// must be the one declaring `property`.
    fn nest(parent: &mut ObjectNode, property: &str, child: ObjectNode) {
        let (slot, declared) = super::super::hierarchy::containment_slot(
            SchemaSet::core().unwrap(),
            &parent.class.element,
            &child.class.element,
        )
        .unwrap();
        assert_eq!(declared.local, property);
        parent.children.push(NestedObject {
            slot,
            property: declared,
            object: child,
        });
    }

    /// One MultiSurface entry under the source property `property`.
    fn surface_entry(lod: u8, property: &str, surfaces: Vec<GmlSurface>) -> GeometryEntry {
        GeometryEntry {
            lod,
            property: Some(property.to_owned()),
            owner: Default::default(),
            element: GmlElement::MultiSurface { id: None, surfaces },
        }
    }

    fn wall(id: Option<&str>) -> ObjectNode {
        node(
            "bldg:WallSurface",
            id,
            vec![surface_entry(
                2,
                "lod2MultiSurface",
                vec![plain_surface(triangle())],
            )],
        )
    }

    /// Write `object` as one tree against the core schema, then its
    /// appearances. Returns the XML and the outcome.
    fn write_tree(
        object: ObjectNode,
        appearances: &[Option<&AppearanceBundle>],
    ) -> (String, WriteOutcome) {
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        let outcome = w.write_object_tree(object, appearances).unwrap();
        w.flush_appearances().unwrap();
        (String::from_utf8(buf).unwrap(), outcome)
    }

    /// The text between the start tag of the element whose start tag begins
    /// with `start` and the first `end` tag after it.
    fn inside<'x>(xml: &'x str, start: &str, end: &str) -> &'x str {
        let from = xml.find(start).unwrap_or_else(|| panic!("{start}: {xml}"));
        let body = &xml[from..];
        let open = body.find('>').unwrap() + 1;
        let close = body.find(end).unwrap_or_else(|| panic!("{end}: {xml}"));
        &body[open..close]
    }

    #[test]
    fn a_nested_object_is_written_inside_its_property() {
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        nest(&mut building, "boundedBy", wall(Some("w1")));
        let (xml, _) = write_tree(building, &[None]);
        assert!(
            xml.contains(
                r#"<bldg:boundedBy><bldg:WallSurface gml:id="w1"><bldg:lod2MultiSurface>"#
            ),
            "{xml}"
        );
        assert!(
            xml.contains(
                "</bldg:WallSurface></bldg:boundedBy></bldg:Building></core:cityObjectMember>"
            ),
            "{xml}"
        );
    }

    #[test]
    fn each_nested_object_gets_its_own_wrapper() {
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        nest(&mut building, "boundedBy", wall(Some("w1")));
        nest(&mut building, "boundedBy", wall(Some("w2")));
        let (xml, _) = write_tree(building, &[None]);
        assert_eq!(xml.matches("<bldg:boundedBy>").count(), 2, "{xml}");
        assert_eq!(xml.matches("<bldg:lod2MultiSurface>").count(), 2, "{xml}");
    }

    #[test]
    fn nested_objects_take_schema_order() {
        let schema = SchemaSet::core().unwrap();
        let (placed, _) = crate::file::citygml::placement::place(
            schema,
            &Prefixes::citygml2(),
            &class("bldg:Building").element,
            vec![text_prop("bldg", "measuredHeight", "10.5")],
        );
        // Geometry arrives before the attribute it follows in schema order,
        // and the wall is nested before either.
        let mut building = node(
            "bldg:Building",
            Some("b1"),
            vec![
                surface_entry(3, "lod3MultiSurface", vec![plain_surface(triangle())]),
                GeometryEntry {
                    lod: 2,
                    property: Some("lod2Solid".to_owned()),
                    owner: Default::default(),
                    element: GmlElement::Solid(solid_of(offset_triangle())),
                },
            ],
        );
        nest(&mut building, "boundedBy", wall(Some("w1")));
        building.properties = placed;
        let (xml, _) = write_tree(building, &[None]);

        let height = xml.find("<bldg:measuredHeight").unwrap();
        let solid = xml.find("<bldg:lod2Solid>").unwrap();
        let bounded = xml.find("<bldg:boundedBy>").unwrap();
        let lod3 = xml.find("<bldg:lod3MultiSurface>").unwrap();
        assert!(height < solid && solid < bounded && bounded < lod3, "{xml}");
    }

    #[test]
    fn a_window_is_written_under_its_walls_opening() {
        let mut wall = wall(Some("w1"));
        nest(
            &mut wall,
            "opening",
            node(
                "bldg:Window",
                Some("win1"),
                vec![surface_entry(
                    3,
                    "lod3MultiSurface",
                    vec![plain_surface(offset_triangle())],
                )],
            ),
        );
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        nest(&mut building, "boundedBy", wall);
        let (xml, _) = write_tree(building, &[None]);

        let in_wall = inside(&xml, "<bldg:WallSurface ", "</bldg:WallSurface>");
        assert!(
            in_wall.contains(r#"<bldg:opening><bldg:Window gml:id="win1"><bldg:lod3MultiSurface>"#),
            "{xml}"
        );
        assert!(in_wall.contains("</bldg:Window></bldg:opening>"), "{xml}");
    }

    #[test]
    fn a_nested_object_without_an_id_gets_a_minted_one() {
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        nest(&mut building, "boundedBy", wall(None));
        let (xml, _) = write_tree(building, &[None]);
        assert!(
            xml.contains(r#"<bldg:WallSurface gml:id="wallsurface_1">"#),
            "{xml}"
        );
    }

    #[test]
    fn a_nested_id_already_used_is_replaced() {
        let mut building = node("bldg:Building", Some("x"), Vec::new());
        nest(&mut building, "boundedBy", wall(Some("x")));
        let (xml, _) = write_tree(building, &[None]);
        assert!(xml.contains(r#"<bldg:Building gml:id="x">"#), "{xml}");
        assert!(
            xml.contains(r#"<bldg:WallSurface gml:id="wallsurface_1">"#),
            "{xml}"
        );
    }

    #[test]
    fn nested_geometry_follows_the_declared_property_rules() {
        // Two entries naming one property, so the merge step has work to do.
        let entries = || {
            vec![
                surface_entry(4, "lod4Geometry", vec![plain_surface(triangle())]),
                surface_entry(4, "lod4Geometry", vec![plain_surface(offset_triangle())]),
            ]
        };
        let mut room = node("bldg:Room", Some("r1"), Vec::new());
        nest(
            &mut room,
            "interiorFurniture",
            node("bldg:BuildingFurniture", Some("f1"), entries()),
        );
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        nest(&mut building, "interiorRoom", room);
        let (nested_xml, _) = write_tree(building, &[None]);
        let (top_xml, _) = write_tree(
            node("bldg:BuildingFurniture", Some("f1"), entries()),
            &[None],
        );

        let nested = inside(
            &nested_xml,
            "<bldg:BuildingFurniture ",
            "</bldg:BuildingFurniture>",
        );
        let top = inside(
            &top_xml,
            "<bldg:BuildingFurniture ",
            "</bldg:BuildingFurniture>",
        );
        assert!(
            nested.starts_with("<bldg:lod4Geometry><gml:MultiSurface"),
            "{nested_xml}"
        );
        assert_eq!(
            nested.matches("<bldg:lod4Geometry>").count(),
            1,
            "{nested_xml}"
        );
        assert_eq!(nested, top);
    }

    #[test]
    fn the_lod1_shell_rule_applies_to_a_nested_building_part() {
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        // From a second source, so the drop is counted against the part's
        // own source rather than the building's.
        let mut part = node("bldg:BuildingPart", Some("p1"), shell_entries(1));
        part.source = 1;
        nest(&mut building, "consistsOfBuildingPart", part);
        let mut buf = Vec::new();
        let mut w = CityGmlXmlWriter::new(&mut buf, false, SRS.to_string());
        let outcome = w.write_object_tree(building, &[None, None]).unwrap();
        assert_eq!(w.dropped_lod1_surfaces(), 1);
        let xml = String::from_utf8(buf).unwrap();

        assert!(!xml.contains("lod1MultiSurface"), "{xml}");
        assert!(
            inside(&xml, "<bldg:BuildingPart ", "</bldg:BuildingPart>")
                .contains("<bldg:lod1Solid>"),
            "{xml}"
        );
        assert_eq!(outcome.lod1_surfaces_dropped, BTreeMap::from([(1, 1)]));
    }

    #[test]
    fn a_nested_walls_textures_resolve() {
        let textured = GmlSurface {
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
        let mut building = node("bldg:Building", Some("b1"), Vec::new());
        nest(
            &mut building,
            "boundedBy",
            node(
                "bldg:WallSurface",
                Some("w1"),
                vec![surface_entry(2, "lod2MultiSurface", vec![textured])],
            ),
        );
        let (xml, _) = write_tree(building, &[Some(&appearance)]);

        assert!(
            inside(&xml, "<bldg:WallSurface ", "</bldg:WallSurface>")
                .contains(r#"<gml:Polygon gml:id="poly_1""#),
            "{xml}"
        );
        assert!(xml.contains(r##"app:target uri="#poly_1""##), "{xml}");
        assert_eq!(xml.matches("<app:appearanceMember>").count(), 1, "{xml}");
    }

    #[test]
    fn objects_from_two_sources_keep_their_own_appearance() {
        let coloured = |surface: Vec<[f64; 3]>| GmlSurface {
            material_idx: Some(0),
            ..plain_surface(surface)
        };
        let bundle = |r: f64, b: f64| {
            let mut material = X3DMaterial::default();
            material.diffuse_color.r = r;
            material.diffuse_color.g = 0.0;
            material.diffuse_color.b = b;
            AppearanceBundle {
                theme: None,
                materials: vec![material],
                textures: vec![],
            }
        };
        let red = bundle(1.0, 0.0);
        let blue = bundle(0.0, 1.0);

        let mut building = node(
            "bldg:Building",
            Some("b1"),
            vec![surface_entry(
                2,
                "lod2MultiSurface",
                vec![coloured(triangle())],
            )],
        );
        let mut part = node(
            "bldg:BuildingPart",
            Some("p1"),
            vec![surface_entry(
                2,
                "lod2MultiSurface",
                vec![coloured(offset_triangle())],
            )],
        );
        part.source = 1;
        nest(&mut building, "consistsOfBuildingPart", part);
        let (xml, _) = write_tree(building, &[Some(&red), Some(&blue)]);

        assert_eq!(xml.matches("<app:appearanceMember>").count(), 2, "{xml}");
        let (first, second) = xml.split_once("</app:appearanceMember>").unwrap();
        let first = first.split_once("<app:appearanceMember>").unwrap().1;
        assert!(
            first.contains("<app:diffuseColor>1 0 0</app:diffuseColor>")
                && first.contains("<app:target>#poly_1</app:target>"),
            "{xml}"
        );
        assert!(
            second.contains("<app:diffuseColor>0 0 1</app:diffuseColor>")
                && second.contains("<app:target>#poly_2</app:target>"),
            "{xml}"
        );
    }

    #[test]
    fn a_relief_component_is_written_under_dem_relief_component() {
        let mut relief = node("dem:ReliefFeature", Some("r1"), Vec::new());
        nest(
            &mut relief,
            "reliefComponent",
            node(
                "dem:BreaklineRelief",
                None,
                vec![GeometryEntry {
                    lod: 2,
                    property: Some("breaklines".to_owned()),
                    owner: Default::default(),
                    element: GmlElement::MultiCurve {
                        id: None,
                        curves: vec![triangle()],
                    },
                }],
            ),
        );
        let (xml, _) = write_tree(relief, &[None]);
        assert!(
            xml.contains("<dem:reliefComponent><dem:BreaklineRelief gml:id="),
            "{xml}"
        );
        assert!(
            inside(&xml, "<dem:BreaklineRelief ", "</dem:BreaklineRelief>")
                .contains("<dem:breaklines><gml:MultiCurve"),
            "{xml}"
        );
    }

    #[test]
    fn a_traffic_area_is_written_under_tran_traffic_area() {
        let mut road = node("tran:Road", Some("road1"), Vec::new());
        nest(
            &mut road,
            "trafficArea",
            node(
                "tran:TrafficArea",
                Some("ta1"),
                vec![surface_entry(
                    2,
                    "lod2MultiSurface",
                    vec![plain_surface(triangle())],
                )],
            ),
        );
        let (xml, _) = write_tree(road, &[None]);
        assert!(
            xml.contains(
                r#"<tran:trafficArea><tran:TrafficArea gml:id="ta1"><tran:lod2MultiSurface>"#
            ),
            "{xml}"
        );
    }
}
