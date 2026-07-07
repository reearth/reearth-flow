//! CityGML appearance parsing for the new-geometry reader.
//!
//! CityGML appearances reference geometry by `gml:id`, the reverse of Flow's
//! geometry-owns-appearance model. This module parses `app:appearanceMember`s into
//! an [`AppearanceIndex`] keyed by the target `gml:id`, so pass-2 resolution can
//! look a face's appearance up from the ids captured with its geometry and attach
//! it at the surface, letting mesh welding carry it across.

use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::appearance::{
    ChannelId, Material, PhongMaterial, Raster, Sampler, Side, Texture, ThemeId, UvSource,
};
use reearth_flow_geometry::polygon::{Polygon3D, PolygonFace};
use url::Url;

use super::parser::{RawChild, RawNode};
use super::utils::local_name;

/// CityGML appearance indexed by the geometry `gml:id` it targets.
#[derive(Default)]
pub(super) struct AppearanceIndex {
    /// Per surface gml:id: the styles bound to it, at most one per theme.
    surfaces: HashMap<String, Vec<SurfaceStyle>>,
    /// Per `(theme, side, ring gml:id)`: that ring's texture coordinates.
    ring_uv: HashMap<(String, Side, String), Vec<[f64; 2]>>,
}

/// One theme's front / back materials for a surface. CityGML `app:isFront`
/// separates the two sides; either may be absent.
struct SurfaceStyle {
    theme: String,
    front: Option<SideMaterial>,
    back: Option<SideMaterial>,
}

/// A material for one side, and whether it samples a texture (so per-ring UV must
/// be assembled when attached).
struct SideMaterial {
    material: Material,
    textured: bool,
}

/// One side's resolved material and UV, ready to attach.
struct ResolvedSide {
    material: Material,
    uv: Option<UvSource>,
}

impl AppearanceIndex {
    /// Whether no appearance was parsed, so resolution can skip attachment.
    pub(super) fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    /// Attach the appearance targeting `polygon` to it, assembling per-ring UV from
    /// `rings` (exterior-first, holes-next, matching the polygon's coords).
    ///
    /// `candidates` are the gml:ids that may carry this polygon's style, most
    /// specific first: the polygon's own surface id, then its enclosing container
    /// ids (a `MultiSurface` / `Solid` may bind a material to a whole aggregate).
    /// The first candidate that carries any style wins, so a leaf's own material
    /// overrides an inherited container material. A textured style whose rings lack
    /// complete UV is skipped rather than attached with a mismatched UV length.
    pub(super) fn apply_to_polygon(
        &self,
        polygon: &mut Polygon3D,
        candidates: &[&str],
        rings: &[Option<String>],
    ) {
        let Some((surface, styles)) = candidates
            .iter()
            .find_map(|id| self.surfaces.get(*id).map(|styles| (*id, styles)))
        else {
            return;
        };
        for style in styles {
            let theme = ThemeId(Arc::from(style.theme.as_str()));
            let front = style
                .front
                .as_ref()
                .and_then(|m| self.resolve_side(m, &style.theme, Side::Front, rings));
            let back = style
                .back
                .as_ref()
                .and_then(|m| self.resolve_side(m, &style.theme, Side::Back, rings));
            let result = match (front, back) {
                (Some(f), Some(b)) => polygon.set_two_sided_appearance(
                    theme,
                    PolygonFace::single(f.material, f.uv),
                    PolygonFace::single(b.material, b.uv),
                ),
                // A lone side (usually the front) becomes a single-sided appearance.
                (Some(s), None) | (None, Some(s)) => {
                    polygon.set_appearance(theme, s.material, s.uv)
                }
                (None, None) => continue,
            };
            if let Err(e) = result {
                tracing::warn!("citygml appearance: could not attach to surface {surface}: {e}");
            }
        }
    }

    /// Resolve one side's material and, for a texture, its assembled per-ring UV.
    /// `None` when a textured side lacks complete UV, so it is skipped rather than
    /// attached with a mismatched UV length.
    fn resolve_side(
        &self,
        side_material: &SideMaterial,
        theme: &str,
        side: Side,
        rings: &[Option<String>],
    ) -> Option<ResolvedSide> {
        let uv = if side_material.textured {
            Some(UvSource::Explicit(
                self.ring_uv_for(theme, side, rings)?.into_boxed_slice(),
            ))
        } else {
            None
        };
        Some(ResolvedSide {
            material: side_material.material.clone(),
            uv,
        })
    }

    /// Concatenate the texture coordinates of every `rings` entry under
    /// `(theme, side)`, in order. `None` if any ring lacks an id or its UV, so a
    /// partial UV never mismatches the polygon's corner count.
    fn ring_uv_for(
        &self,
        theme: &str,
        side: Side,
        rings: &[Option<String>],
    ) -> Option<Vec<[f64; 2]>> {
        let mut out = Vec::new();
        for ring in rings {
            let ring = ring.as_ref()?;
            let uv = self.ring_uv.get(&(theme.to_string(), side, ring.clone()))?;
            out.extend_from_slice(uv);
        }
        Some(out)
    }
}

/// Index every `app:Appearance` under an `app:appearanceMember` element.
pub(super) fn index_appearance_member(member: &RawNode, index: &mut AppearanceIndex) {
    for appearance in child_elements(member) {
        if local_name(&appearance.name.0) == "Appearance" {
            index_appearance(appearance, index);
        }
    }
}

/// Index one `app:Appearance`: its theme's `app:ParameterizedTexture`s and
/// `app:X3DMaterial`s. The surface-data property is `surfaceData` in CityGML 3.0
/// and `surfaceDataMember` in 2.0.
fn index_appearance(appearance: &RawNode, index: &mut AppearanceIndex) {
    let theme = child_text(appearance, "theme").unwrap_or_default();
    for member in child_elements(appearance) {
        if !matches!(
            local_name(&member.name.0),
            "surfaceData" | "surfaceDataMember"
        ) {
            continue;
        }
        for data in child_elements(member) {
            match local_name(&data.name.0) {
                "ParameterizedTexture" => index_texture(data, &theme, index),
                "X3DMaterial" => index_material(data, &theme, index),
                // TODO: GeoreferencedTexture / TexCoordGen (worldToTexture).
                _ => {}
            }
        }
    }
}

/// The side an `app:_SurfaceData` paints, from its `app:isFront` (default front).
fn side_of(surface_data: &RawNode) -> Side {
    match child_text(surface_data, "isFront").as_deref() {
        Some("false") | Some("0") => Side::Back,
        _ => Side::Front,
    }
}

/// Bind `side_material` to one side of `surface` under `theme`, keeping one
/// material per (theme, side). A textured material outranks a colour-only one,
/// regardless of document order.
fn set_side(index: &mut AppearanceIndex, surface: &str, theme: &str, side: Side, sm: SideMaterial) {
    let styles = index.surfaces.entry(surface.to_string()).or_default();
    let i = match styles.iter().position(|s| s.theme == theme) {
        Some(i) => i,
        None => {
            styles.push(SurfaceStyle {
                theme: theme.to_string(),
                front: None,
                back: None,
            });
            styles.len() - 1
        }
    };
    let slot = match side {
        Side::Front => &mut styles[i].front,
        Side::Back => &mut styles[i].back,
    };
    let replace = match slot {
        None => true,
        Some(existing) => sm.textured && !existing.textured,
    };
    if replace {
        *slot = Some(sm);
    }
}

/// Index one `app:ParameterizedTexture`: its image as a textured material bound to
/// each target surface's side, and each target ring's texture coordinates. Handles
/// both the CityGML 2.0 shape (`target` with a `uri` attribute, `textureCoordinates`
/// with a `ring` attribute) and the 3.0 shape (`textureParameterization` of
/// `TextureAssociation`, with element-text `target` / `ring`).
fn index_texture(texture: &RawNode, theme: &str, index: &mut AppearanceIndex) {
    let Some(image) = child_text(texture, "imageURI") else {
        return;
    };
    let Some(uri) = resolve_uri(&texture.source_url, &image) else {
        return;
    };
    let side = side_of(texture);
    let raster = Arc::new(Raster::Uri(uri));
    for child in child_elements(texture) {
        match local_name(&child.name.0) {
            "target" => index_texture_target_v2(child, theme, side, &raster, index),
            "textureParameterization" => {
                for assoc in child_elements(child) {
                    if local_name(&assoc.name.0) == "TextureAssociation" {
                        index_texture_target_v3(assoc, theme, side, &raster, index);
                    }
                }
            }
            _ => {}
        }
    }
}

/// A textured material for one side backed by `raster`.
fn textured_side(raster: &Arc<Raster>) -> SideMaterial {
    SideMaterial {
        material: texture_material(Arc::clone(raster)),
        textured: true,
    }
}

/// CityGML 2.0 texture target: `<app:target uri="#surface">` wrapping a
/// `TexCoordList` of `<app:textureCoordinates ring="#ring">`.
fn index_texture_target_v2(
    target: &RawNode,
    theme: &str,
    side: Side,
    raster: &Arc<Raster>,
    index: &mut AppearanceIndex,
) {
    let Some(surface) = attr(target, "uri").map(strip_hash) else {
        return;
    };
    set_side(index, surface, theme, side, textured_side(raster));
    for coord_list in child_elements(target) {
        if local_name(&coord_list.name.0) != "TexCoordList" {
            continue;
        }
        for tc in child_elements(coord_list) {
            if local_name(&tc.name.0) != "textureCoordinates" {
                continue;
            }
            let Some(ring) = attr(tc, "ring").map(strip_hash) else {
                continue;
            };
            if let Some(uv) = parse_uv(&text_of(tc)) {
                index
                    .ring_uv
                    .insert((theme.to_string(), side, ring.to_string()), uv);
            }
        }
    }
}

/// CityGML 3.0 texture target: an `<app:TextureAssociation>` with an element-text
/// `<app:target>#surface` and a nested `textureParameterization` / `TexCoordList`
/// whose `textureCoordinates` and `ring` are positional siblings.
fn index_texture_target_v3(
    assoc: &RawNode,
    theme: &str,
    side: Side,
    raster: &Arc<Raster>,
    index: &mut AppearanceIndex,
) {
    let Some(surface) = child_text(assoc, "target") else {
        return;
    };
    let surface = strip_hash(&surface);
    set_side(index, surface, theme, side, textured_side(raster));
    for param in child_elements(assoc) {
        if local_name(&param.name.0) != "textureParameterization" {
            continue;
        }
        for coord_list in child_elements(param) {
            if local_name(&coord_list.name.0) != "TexCoordList" {
                continue;
            }
            for (ring, uv) in tex_coord_pairs(coord_list) {
                index.ring_uv.insert((theme.to_string(), side, ring), uv);
            }
        }
    }
}

/// Pair a CityGML 3.0 `TexCoordList`'s `textureCoordinates` with its `ring`s by
/// document order: the i-th ring gets the i-th coordinate list. An unparseable
/// coordinate list keeps its slot (empty) so later pairs stay aligned; it is
/// dropped later when the UV length fails to match the ring.
fn tex_coord_pairs(coord_list: &RawNode) -> Vec<(String, Vec<[f64; 2]>)> {
    let mut coords: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut rings: Vec<String> = Vec::new();
    for child in child_elements(coord_list) {
        match local_name(&child.name.0) {
            "textureCoordinates" => coords.push(parse_uv(&text_of(child)).unwrap_or_default()),
            "ring" => rings.push(strip_hash(text_of(child).trim()).to_string()),
            _ => {}
        }
    }
    rings.into_iter().zip(coords).collect()
}

/// A white Phong material whose diffuse map is `raster` on the default UV channel.
fn texture_material(raster: Arc<Raster>) -> Material {
    Material::Phong(PhongMaterial {
        diffuse: [1.0, 1.0, 1.0],
        specular: [0.0; 3],
        emissive: [0.0; 3],
        ambient_intensity: 0.0,
        shininess: 0.0,
        transparency: 0.0,
        diffuse_map: Some(Texture {
            raster,
            sampler: Sampler::default(),
            transform: None,
            uv_channel: ChannelId::default(),
        }),
        emissive_map: None,
        normal_map: None,
    })
}

/// Index one `app:X3DMaterial`: a colour-only Phong material bound to each target
/// surface. Its `app:target`s are element text (`#surface-id`), not the `uri`
/// attribute a `ParameterizedTexture` uses.
fn index_material(material_node: &RawNode, theme: &str, index: &mut AppearanceIndex) {
    let material = Material::Phong(PhongMaterial {
        diffuse: child_color(material_node, "diffuseColor", [0.8, 0.8, 0.8]),
        specular: child_color(material_node, "specularColor", [1.0, 1.0, 1.0]),
        emissive: child_color(material_node, "emissiveColor", [0.0, 0.0, 0.0]),
        ambient_intensity: child_f32(material_node, "ambientIntensity", 0.2),
        shininess: child_f32(material_node, "shininess", 0.2),
        transparency: child_f32(material_node, "transparency", 0.0),
        diffuse_map: None,
        emissive_map: None,
        normal_map: None,
    });
    let side = side_of(material_node);
    for target in child_elements(material_node) {
        if local_name(&target.name.0) != "target" {
            continue;
        }
        let text = text_of(target);
        let surface = strip_hash(text.trim());
        if surface.is_empty() {
            continue;
        }
        set_side(
            index,
            surface,
            theme,
            side,
            SideMaterial {
                material: material.clone(),
                textured: false,
            },
        );
    }
}

/// A material colour element (three floats), or `default` when absent or malformed.
fn child_color(node: &RawNode, name: &str, default: [f32; 3]) -> [f32; 3] {
    let Some(text) = child_text(node, name) else {
        return default;
    };
    let values: Vec<f32> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    match values[..] {
        [r, g, b] => [r, g, b],
        _ => default,
    }
}

/// A single-float material element, or `default` when absent or malformed.
fn child_f32(node: &RawNode, name: &str, default: f32) -> f32 {
    child_text(node, name)
        .and_then(|t| t.parse().ok())
        .unwrap_or(default)
}

/// Resolve a relative `app:imageURI` against the source document into a scheme-aware
/// [`Uri`] pointing at the image's own location.
fn resolve_uri(source_url: &Url, image: &str) -> Option<Uri> {
    let joined = source_url.join(image).ok()?;
    match Uri::try_from(joined) {
        Ok(uri) => Some(uri),
        Err(e) => {
            tracing::warn!("citygml appearance: invalid image uri {image}: {e}");
            None
        }
    }
}

/// Parse whitespace-separated `u v` pairs; `None` if any token is unparseable or
/// the count is not even.
fn parse_uv(text: &str) -> Option<Vec<[f64; 2]>> {
    let values: Option<Vec<f64>> = text
        .split_whitespace()
        .map(|s| s.parse::<f64>().ok())
        .collect();
    let values = values?;
    if values.is_empty() || !values.len().is_multiple_of(2) {
        return None;
    }
    Some(values.chunks_exact(2).map(|c| [c[0], c[1]]).collect())
}

/// Iterate a node's element children.
fn child_elements(node: &RawNode) -> impl Iterator<Item = &RawNode> {
    node.children.iter().filter_map(|c| match c {
        RawChild::Element(e) => Some(e.as_ref()),
        _ => None,
    })
}

/// The trimmed text of the first child element named `name`, if non-empty.
fn child_text(node: &RawNode, name: &str) -> Option<String> {
    child_elements(node)
        .find(|e| local_name(&e.name.0) == name)
        .map(|e| text_of(e).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// A node's concatenated text content.
fn text_of(node: &RawNode) -> String {
    node.children
        .iter()
        .filter_map(|c| match c {
            RawChild::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

/// The value of the attribute named `name`, ignoring namespace.
fn attr<'a>(node: &'a RawNode, name: &str) -> Option<&'a str> {
    node.attrs
        .iter()
        .find(|((q, _), _)| local_name(q) == name)
        .map(|(_, v)| v.as_str())
}

/// Strip a leading `#` from an `xlink`-style local reference.
fn strip_hash(reference: &str) -> &str {
    reference.strip_prefix('#').unwrap_or(reference)
}

#[cfg(test)]
mod tests {
    use crate::citygml_parser::parser::Parser;
    use crate::citygml_parser::resolver::resolve_root_with_appearance;
    use reearth_flow_geometry::appearance::{Material, Side, ThemeId, UvSource};
    use reearth_flow_geometry::Euclidean3DGeometry;
    use std::sync::Arc;
    use url::Url;

    const NS: &str = r#"xmlns:core="http://www.opengis.net/citygml/3.0"
        xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
        xmlns:gml="http://www.opengis.net/gml/3.2"
        xmlns:app="http://www.opengis.net/citygml/appearance/3.0"
        xmlns:xlink="http://www.w3.org/1999/xlink""#;

    /// The sole `Polygon` of a feature whose geometry is a one-member collection of
    /// one polygon.
    fn resolve_only_polygon(xml: &str) -> Polygon3DOut {
        let mut parser = Parser::new();
        parser
            .parse(xml.as_bytes(), &Url::parse("file:///dir/test.gml").unwrap())
            .unwrap();
        let (pending, _raw, geom_registry, appearance, _ns) = parser.finish();
        assert!(!appearance.is_empty(), "appearance should be indexed");
        let feature = pending.into_iter().next().expect("one feature");
        let geom =
            resolve_root_with_appearance(&feature.geoms[0].node, &geom_registry, &appearance)
                .expect("geometry resolves");
        match geom {
            Euclidean3DGeometry::Collection(c) => match c.members().first().expect("one member") {
                Euclidean3DGeometry::Polygon(p) => Polygon3DOut((**p).clone()),
                other => panic!("expected Polygon, got {other:?}"),
            },
            other => panic!("expected Collection, got {other:?}"),
        }
    }

    struct Polygon3DOut(reearth_flow_geometry::polygon::Polygon3D);

    #[test]
    fn parameterized_texture_attaches_to_polygon() {
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <app:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceDataMember><app:ParameterizedTexture>
                  <app:imageURI>tex/a.jpg</app:imageURI>
                  <app:target uri="#poly1"><app:TexCoordList>
                    <app:textureCoordinates ring="#ring1">0 0 1 0 1 1 0 0</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceDataMember>
              </app:Appearance></app:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon is textured");
        assert_eq!(appearance.default_theme, ThemeId(Arc::from("rgbTexture")));
        assert!(matches!(
            &appearance.materials[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        let uv = polygon.uv_sets();
        assert_eq!(uv.len(), 1);
        let UvSource::Explicit(coords) = &uv[0].uv else {
            panic!("expected explicit UV");
        };
        // The ring's four (closed) vertices each carry a UV pair.
        assert_eq!(coords.len(), 4);
        assert_eq!(coords[1], [1.0, 0.0]);
    }

    #[test]
    fn texture_missing_ring_uv_leaves_polygon_bare() {
        // The texture targets the surface but references a ring id the polygon does
        // not carry, so no full UV can be assembled and nothing is attached.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <app:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceDataMember><app:ParameterizedTexture>
                  <app:imageURI>tex/a.jpg</app:imageURI>
                  <app:target uri="#poly1"><app:TexCoordList>
                    <app:textureCoordinates ring="#other">0 0 1 0 1 1 0 0</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceDataMember>
              </app:Appearance></app:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        assert!(polygon.appearance().is_none());
        assert!(polygon.uv_sets().is_empty());
    }

    #[test]
    fn x3d_material_colour_attaches_to_polygon() {
        // An X3DMaterial targets the surface by element text (`#poly1`), carries no
        // UV, and yields a colour-only Phong material.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <app:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceDataMember><app:X3DMaterial>
                  <app:diffuseColor>0.588235 0.588235 0.588235</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceDataMember>
              </app:Appearance></app:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon is coloured");
        let Material::Phong(m) = &appearance.materials[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.588235, 0.588235, 0.588235]);
        assert!(m.diffuse_map.is_none(), "colour-only, no texture");
        assert!(polygon.uv_sets().is_empty(), "colour-only, no UV");
    }

    #[test]
    fn texture_outranks_colour_on_same_surface() {
        // The X3DMaterial is listed first, but the ParameterizedTexture for the same
        // surface + theme must win regardless of document order.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <app:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceDataMember><app:X3DMaterial>
                  <app:diffuseColor>0.1 0.2 0.3</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceDataMember>
                <app:surfaceDataMember><app:ParameterizedTexture>
                  <app:imageURI>tex/a.jpg</app:imageURI>
                  <app:target uri="#poly1"><app:TexCoordList>
                    <app:textureCoordinates ring="#ring1">0 0 1 0 1 1 0 0</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceDataMember>
              </app:Appearance></app:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon styled");
        assert_eq!(appearance.materials.len(), 1, "one style wins, not both");
        assert!(matches!(
            &appearance.materials[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        assert_eq!(polygon.uv_sets().len(), 1, "the texture's UV is present");
    }

    #[test]
    fn cg3_surface_data_element_is_recognized() {
        // CityGML 3.0 wraps surface data in `app:surfaceData` (not the 2.0
        // `surfaceDataMember`) under a `core:appearanceMember`.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0.2 0.4 0.6</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon
            .appearance()
            .as_ref()
            .expect("surfaceData recognized");
        let Material::Phong(m) = &appearance.materials[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.2, 0.4, 0.6]);
    }

    #[test]
    fn two_sided_isfront_attaches_front_and_back() {
        // A front (isFront=true) and back (isFront=false) X3DMaterial on one surface
        // become a two-sided appearance: two materials, a front and a back binding.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:isFront>true</app:isFront>
                  <app:diffuseColor>1 0 0</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
                <app:surfaceData><app:X3DMaterial>
                  <app:isFront>false</app:isFront>
                  <app:diffuseColor>0 0 1</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon styled");
        assert_eq!(appearance.materials.len(), 2, "front and back materials");
        assert!(
            appearance.themes[0].back.is_some(),
            "the back side is bound"
        );
        let colours: Vec<[f32; 3]> = appearance
            .materials
            .iter()
            .map(|m| match m {
                Material::Phong(p) => p.diffuse,
                _ => panic!("expected Phong"),
            })
            .collect();
        assert!(colours.contains(&[1.0, 0.0, 0.0]));
        assert!(colours.contains(&[0.0, 0.0, 1.0]));
    }

    #[test]
    fn two_sided_texture_keeps_uv_per_side() {
        // Front and back textures on one surface carry distinct per-ring UV; both
        // sides' UV sets survive, keyed by side.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:ParameterizedTexture>
                  <app:isFront>true</app:isFront>
                  <app:imageURI>tex/front.jpg</app:imageURI>
                  <app:target uri="#poly1"><app:TexCoordList>
                    <app:textureCoordinates ring="#ring1">0 0 1 0 1 1 0 0</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceData>
                <app:surfaceData><app:ParameterizedTexture>
                  <app:isFront>false</app:isFront>
                  <app:imageURI>tex/back.jpg</app:imageURI>
                  <app:target uri="#poly1"><app:TexCoordList>
                    <app:textureCoordinates ring="#ring1">0.1 0.1 0.2 0.1 0.2 0.2 0.1 0.1</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let uv = polygon.uv_sets();
        assert_eq!(uv.len(), 2, "one UV set per side");
        assert!(uv.iter().any(|s| s.side == Side::Front));
        assert!(uv.iter().any(|s| s.side == Side::Back));
        let back = uv.iter().find(|s| s.side == Side::Back).unwrap();
        let UvSource::Explicit(coords) = &back.uv else {
            panic!("explicit");
        };
        assert_eq!(coords[0], [0.1, 0.1], "back side keeps its own UV");
    }

    #[test]
    fn container_material_attaches_to_member_polygon() {
        // The X3DMaterial targets the MultiSurface container (`#ms1`), not the
        // polygon. Its member polygon, itself untargeted, inherits the material.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface gml:id="ms1"><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0.1 0.2 0.3</app:diffuseColor>
                  <app:target>#ms1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon
            .appearance()
            .as_ref()
            .expect("member inherits container material");
        let Material::Phong(m) = &appearance.materials[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.1, 0.2, 0.3]);
    }

    #[test]
    fn leaf_material_overrides_container_material() {
        // The polygon is targeted directly (`#poly1`) and via its container
        // (`#ms1`). The more specific leaf material wins.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface gml:id="ms1"><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0.9 0.9 0.9</app:diffuseColor>
                  <app:target>#ms1</app:target>
                </app:X3DMaterial></app:surfaceData>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0.1 0.1 0.1</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon styled");
        assert_eq!(
            appearance.materials.len(),
            1,
            "only the leaf material attaches"
        );
        let Material::Phong(m) = &appearance.materials[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.1, 0.1, 0.1], "leaf wins over container");
    }

    #[test]
    fn cg3_parameterized_texture_attaches_to_polygon() {
        // CityGML 3.0 texture shape: textureParameterization / TextureAssociation,
        // element-text target and ring, positional textureCoordinates + ring.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:ParameterizedTexture>
                  <app:imageURI>tex/a.jpg</app:imageURI>
                  <app:textureParameterization><app:TextureAssociation>
                    <app:target>#poly1</app:target>
                    <app:textureParameterization><app:TexCoordList>
                      <app:textureCoordinates>0 0 1 0 1 1 0 0</app:textureCoordinates>
                      <app:ring>#ring1</app:ring>
                    </app:TexCoordList></app:textureParameterization>
                  </app:TextureAssociation></app:textureParameterization>
                </app:ParameterizedTexture></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon is textured");
        assert!(matches!(
            &appearance.materials[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        let uv = polygon.uv_sets();
        assert_eq!(uv.len(), 1);
        let UvSource::Explicit(coords) = &uv[0].uv else {
            panic!("expected explicit UV");
        };
        assert_eq!(coords.len(), 4);
        assert_eq!(coords[1], [1.0, 0.0]);
    }
}
