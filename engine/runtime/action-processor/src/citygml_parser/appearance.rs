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
    ChannelId, Material, PhongMaterial, Raster, Sampler, Texture, ThemeId, UvSource,
};
use reearth_flow_geometry::polygon::Polygon3D;
use url::Url;

use super::parser::{RawChild, RawNode};
use super::utils::local_name;

/// CityGML appearance indexed by the geometry `gml:id` it targets.
#[derive(Default)]
pub(super) struct AppearanceIndex {
    /// Per surface gml:id: the styles bound to it, at most one per theme.
    surfaces: HashMap<String, Vec<SurfaceStyle>>,
    /// Per `(theme, ring gml:id)`: that ring's texture coordinates.
    ring_uv: HashMap<(String, String), Vec<[f64; 2]>>,
}

/// One theme's material for a surface, and whether it samples a texture (and so
/// needs per-ring UV assembled when attached).
struct SurfaceStyle {
    theme: String,
    material: Material,
    textured: bool,
}

impl AppearanceIndex {
    /// Whether no appearance was parsed, so resolution can skip attachment.
    pub(super) fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    /// Attach the appearance targeting `surface` to `polygon`, assembling per-ring
    /// UV from `rings` (exterior-first, holes-next, matching the polygon's coords).
    /// A textured style whose rings lack complete UV is skipped rather than
    /// attached with a mismatched UV length.
    pub(super) fn apply_to_polygon(
        &self,
        polygon: &mut Polygon3D,
        surface: Option<&str>,
        rings: &[Option<String>],
    ) {
        let Some(surface) = surface else {
            return;
        };
        let Some(styles) = self.surfaces.get(surface) else {
            return;
        };
        for style in styles {
            let uv = if style.textured {
                match self.ring_uv_for(&style.theme, rings) {
                    Some(uv) => Some(UvSource::Explicit(uv.into_boxed_slice())),
                    None => continue,
                }
            } else {
                None
            };
            if let Err(e) = polygon.set_appearance(
                ThemeId(Arc::from(style.theme.as_str())),
                style.material.clone(),
                uv,
            ) {
                tracing::warn!("citygml appearance: could not attach to surface {surface}: {e}");
            }
        }
    }

    /// Concatenate the texture coordinates of every `rings` entry under `theme`, in
    /// order. `None` if any ring lacks an id or its UV, so a partial UV never
    /// mismatches the polygon's corner count.
    fn ring_uv_for(&self, theme: &str, rings: &[Option<String>]) -> Option<Vec<[f64; 2]>> {
        let mut out = Vec::new();
        for ring in rings {
            let ring = ring.as_ref()?;
            let uv = self.ring_uv.get(&(theme.to_string(), ring.clone()))?;
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
/// `app:X3DMaterial`s.
fn index_appearance(appearance: &RawNode, index: &mut AppearanceIndex) {
    let theme = child_text(appearance, "theme").unwrap_or_default();
    for member in child_elements(appearance) {
        if local_name(&member.name.0) != "surfaceDataMember" {
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

/// Record `style` for `surface`, keeping one style per theme. A textured style
/// outranks a colour-only one under the same theme, regardless of document order.
fn insert_style(index: &mut AppearanceIndex, surface: &str, style: SurfaceStyle) {
    let styles = index.surfaces.entry(surface.to_string()).or_default();
    match styles.iter().position(|s| s.theme == style.theme) {
        Some(i) => {
            if style.textured && !styles[i].textured {
                styles[i] = style;
            }
        }
        None => styles.push(style),
    }
}

/// Index one `app:ParameterizedTexture`: its image as a textured material for each
/// target surface, and each target ring's texture coordinates.
fn index_texture(texture: &RawNode, theme: &str, index: &mut AppearanceIndex) {
    let Some(image) = child_text(texture, "imageURI") else {
        return;
    };
    let Some(uri) = resolve_uri(&texture.source_url, &image) else {
        return;
    };
    let raster = Arc::new(Raster::Uri(uri));
    for target in child_elements(texture) {
        if local_name(&target.name.0) != "target" {
            continue;
        }
        let Some(surface) = attr(target, "uri").map(strip_hash) else {
            continue;
        };
        insert_style(
            index,
            surface,
            SurfaceStyle {
                theme: theme.to_string(),
                material: texture_material(Arc::clone(&raster)),
                textured: true,
            },
        );
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
                        .insert((theme.to_string(), ring.to_string()), uv);
                }
            }
        }
    }
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
    for target in child_elements(material_node) {
        if local_name(&target.name.0) != "target" {
            continue;
        }
        let text = text_of(target);
        let surface = strip_hash(text.trim());
        if surface.is_empty() {
            continue;
        }
        insert_style(
            index,
            surface,
            SurfaceStyle {
                theme: theme.to_string(),
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
    use reearth_flow_geometry::appearance::{Material, ThemeId, UvSource};
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
}
