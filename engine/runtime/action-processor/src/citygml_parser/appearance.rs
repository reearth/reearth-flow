//! CityGML appearance parsing for the new-geometry reader.
//!
//! CityGML appearances reference geometry by `gml:id`, the reverse of Flow's
//! geometry-owns-appearance model. In pass 2, once every `gml:id` is known,
//! [`build_index`] resolves each retained `app:appearanceMember` (following any
//! `xlink:href` to shared surface data or a shared `app:Appearance`) into an
//! [`AppearanceIndex`] keyed by the target `gml:id`, so resolution can look a
//! face's appearance up from the ids captured with its geometry and attach it at
//! the surface, letting mesh welding carry it across.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::appearance::{
    ChannelId, Material, PhongMaterial, Raster, Sampler, Side, Texture, ThemeId, UvSource, WrapMode,
};
use reearth_flow_geometry::polygon::{Polygon3D, PolygonFace};
use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
use url::Url;

use super::parser::{RawNode, RawRegistry};
use super::resolver::FaceIds;
use super::utils::{local_name, XmlChild, XmlNode};
use super::xlink;

/// A geometry id qualified by the source file it belongs to: `(file_url, gml_id)`.
/// An appearance target resolves against the file of the appearance that declares
/// it, mirroring the file-qualified geometry registry, so identical `gml:id`s in
/// different files never cross-bind.
type SurfaceKey = (String, String);

/// CityGML appearance indexed by the file-qualified geometry id it targets.
#[derive(Default)]
pub(super) struct AppearanceIndex {
    /// Per `(file, surface gml:id)`: the styles bound to it, at most one per theme.
    surfaces: HashMap<SurfaceKey, Vec<SurfaceStyle>>,
    /// Per `(file, theme, side, ring gml:id)`: that ring's texture coordinates.
    ring_uv: HashMap<(String, String, Side, String), Vec<[f64; 2]>>,
    /// Unsupported appearance sub-elements already warned about, so each is
    /// reported once per parser run rather than once per occurrence.
    warned: HashSet<&'static str>,
}

/// One theme's front / back materials for a surface. CityGML `app:isFront`
/// separates the two sides; either may be absent.
struct SurfaceStyle {
    theme: String,
    front: Option<SideMaterial>,
    back: Option<SideMaterial>,
}

/// A material for one side, accumulated from the surface data targeting it. The
/// X3DMaterial colour and the ParameterizedTexture (image plus sampler) are kept
/// separately and merged at attach time, so a surface carrying both keeps its
/// colour and its texture. At least one field is set.
#[derive(Default)]
struct SideMaterial {
    /// The X3DMaterial's colour Phong (no diffuse map), if a colour was bound.
    color: Option<PhongMaterial>,
    /// The texture's image and sampler, if a texture was bound.
    texture: Option<(Arc<Raster>, Sampler)>,
}

impl SideMaterial {
    /// Whether a texture was bound, so per-ring UV must be assembled when attached.
    fn is_textured(&self) -> bool {
        self.texture.is_some()
    }

    /// The merged material: the X3DMaterial colour (or white when only a texture
    /// was bound) carrying the bound texture as its diffuse map. When both a colour
    /// and a texture are bound, the texture is modulated by the colour.
    fn resolve(&self) -> Material {
        let mut phong = self.color.clone().unwrap_or_else(white_phong);
        if let Some((raster, sampler)) = &self.texture {
            phong.diffuse_map = Some(Texture {
                raster: Arc::clone(raster),
                sampler: *sampler,
                transform: None,
                uv_channel: ChannelId::default(),
            });
        }
        Material::Phong(phong)
    }
}

/// One surface-data contribution to a side: an X3DMaterial colour or a texture.
enum Contribution {
    Color(PhongMaterial),
    Texture(Arc<Raster>, Sampler),
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

    /// Bind a textured material backed by `raster` and sampled per `sampler` to
    /// `target`'s surface side and record each of its rings' texture coordinates
    /// under `(file, theme, side)`.
    fn add_texture_target(
        &mut self,
        target: TextureTarget,
        theme: &str,
        side: Side,
        raster: &Arc<Raster>,
        sampler: Sampler,
    ) {
        set_side(
            self,
            target.surface,
            theme,
            side,
            Contribution::Texture(Arc::clone(raster), sampler),
        );
        for ((file, ring), uv) in target.rings {
            self.ring_uv
                .insert((file, theme.to_string(), side, ring), uv);
        }
    }

    /// Attach the appearance targeting `polygon` to it, assembling per-ring UV from
    /// `rings` (exterior-first, holes-next, matching the polygon's coords).
    ///
    /// `candidates` are the file-qualified ids that may carry this polygon's style,
    /// most specific first: the polygon's own surface id, then its enclosing
    /// container ids (a `MultiSurface` / `Solid` may bind a material to a whole
    /// aggregate). `ring_file` is the file the polygon's ring ids belong to.
    /// Styles are merged per theme with the most specific candidate winning, so a
    /// leaf's own material overrides an inherited container material for that theme
    /// while a container theme the leaf does not carry is still inherited. A
    /// textured style whose rings lack complete UV is skipped rather than attached
    /// with a mismatched UV length.
    pub(super) fn apply_to_polygon(
        &self,
        polygon: &mut Polygon3D,
        candidates: &[(&str, &str)],
        ring_file: &str,
        rings: &[Option<String>],
    ) {
        for (surface, style) in self.merged_styles(candidates) {
            let theme = ThemeId(Arc::from(style.theme.as_str()));
            let front = style
                .front
                .as_ref()
                .and_then(|m| self.resolve_side(m, ring_file, &style.theme, Side::Front, rings));
            let back = style
                .back
                .as_ref()
                .and_then(|m| self.resolve_side(m, ring_file, &style.theme, Side::Back, rings));
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

    /// The styles bound to `candidates`, deduplicated by theme with the most
    /// specific candidate winning: candidates are tried in order (leaf first, then
    /// enclosing containers), and the first style seen for a theme is kept, so a
    /// leaf's own material overrides an inherited container material per theme while
    /// a container theme absent from the leaf is still inherited. Each entry pairs
    /// the winning candidate's id (for diagnostics) with its style.
    fn merged_styles(&self, candidates: &[(&str, &str)]) -> Vec<(&str, &SurfaceStyle)> {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut out = Vec::new();
        for (file, id) in candidates {
            let key = ((*file).to_string(), (*id).to_string());
            let Some((surface, styles)) = self.surfaces.get_key_value(&key) else {
                continue;
            };
            for style in styles {
                if seen.insert(style.theme.as_str()) {
                    out.push((surface.1.as_str(), style));
                }
            }
        }
        out
    }

    /// Resolve one side's material and, for a texture, its assembled per-ring UV.
    /// `None` when a textured side lacks complete UV, so it is skipped rather than
    /// attached with a mismatched UV length.
    fn resolve_side(
        &self,
        side_material: &SideMaterial,
        ring_file: &str,
        theme: &str,
        side: Side,
        rings: &[Option<String>],
    ) -> Option<ResolvedSide> {
        let uv = if side_material.is_textured() {
            Some(UvSource::Explicit(
                self.ring_uv_for(ring_file, theme, side, rings)?
                    .into_boxed_slice(),
            ))
        } else {
            None
        };
        Some(ResolvedSide {
            material: side_material.resolve(),
            uv,
        })
    }

    /// Concatenate the texture coordinates of every `rings` entry under
    /// `(ring_file, theme, side)`, in order. `None` if any ring lacks an id or its
    /// UV, so a partial UV never mismatches the polygon's corner count.
    fn ring_uv_for(
        &self,
        ring_file: &str,
        theme: &str,
        side: Side,
        rings: &[Option<String>],
    ) -> Option<Vec<[f64; 2]>> {
        let mut out = Vec::new();
        for ring in rings {
            let ring = ring.as_ref()?;
            let uv = self.ring_uv.get(&(
                ring_file.to_string(),
                theme.to_string(),
                side,
                ring.clone(),
            ))?;
            out.extend_from_slice(uv);
        }
        Some(out)
    }

    /// Attach the appearance targeting `mesh` to it, draping one texture over its
    /// triangles: `faces` gives each triangle's ring in triangle order, from which
    /// one UV coordinate per corner (`3 * triangle_count`) is assembled.
    ///
    /// `candidates` are the file-qualified ids that may carry the mesh's style, most
    /// specific first: the surface's own id, then its enclosing container ids.
    /// `ring_file` is the file the mesh's triangle ring ids belong to. Styles are
    /// merged per theme with the most specific candidate winning. A triangular mesh
    /// binds a single side (front preferred, else a lone back); a textured style
    /// whose triangles lack complete UV is skipped rather than attached with a
    /// mismatched UV length.
    pub(super) fn apply_to_triangular_mesh(
        &self,
        mesh: &mut TriangularMesh3D,
        candidates: &[(&str, &str)],
        ring_file: &str,
        faces: &[FaceIds],
    ) {
        for (surface, style) in self.merged_styles(candidates) {
            let (side, side_material) = match (style.front.as_ref(), style.back.as_ref()) {
                (Some(front), _) => (Side::Front, front),
                (None, Some(back)) => (Side::Back, back),
                (None, None) => continue,
            };
            let uv = if side_material.is_textured() {
                match self.mesh_uv(ring_file, &style.theme, side, faces) {
                    Some(uv) => Some(UvSource::Explicit(uv.into_boxed_slice())),
                    None => continue,
                }
            } else {
                None
            };
            let theme = ThemeId(Arc::from(style.theme.as_str()));
            if let Err(e) = mesh.set_appearance(theme, side_material.resolve(), uv) {
                tracing::warn!(
                    "citygml appearance: could not attach to triangulated surface {surface}: {e}"
                );
            }
        }
    }

    /// Concatenate the first three texture coordinates of every triangle's ring
    /// under `(ring_file, theme, side)`, in triangle order, giving `3 *
    /// triangle_count` corners to match the welded soup (each triangle keeps its
    /// first three vertices). `None` if any triangle lacks a ring id, its UV, or a
    /// full three coordinates, so a partial UV never mismatches the mesh corner
    /// count.
    fn mesh_uv(
        &self,
        ring_file: &str,
        theme: &str,
        side: Side,
        faces: &[FaceIds],
    ) -> Option<Vec<[f64; 2]>> {
        let mut out = Vec::with_capacity(faces.len() * 3);
        for face in faces {
            let ring = face.rings.first()?.as_ref()?;
            let uv = self.ring_uv.get(&(
                ring_file.to_string(),
                theme.to_string(),
                side,
                ring.clone(),
            ))?;
            if uv.len() < 3 {
                return None;
            }
            out.extend_from_slice(&uv[..3]);
        }
        Some(out)
    }
}

/// Build the appearance index from the retained `app:appearanceMember` roots,
/// resolving each `xlink:href` (to shared surface data or a shared `app:Appearance`)
/// against `registry` before indexing.
pub(super) fn build_index(members: &[Arc<RawNode>], registry: &RawRegistry) -> AppearanceIndex {
    let mut index = AppearanceIndex::default();
    let mut cache = xlink::ResolveCache::new();
    for member in members {
        if let Some(resolved) = xlink::resolve_one(member, registry, &mut cache) {
            index_appearance_member(&resolved, &mut index);
        }
    }
    index
}

/// Index every `app:Appearance` under an `app:appearanceMember` element.
fn index_appearance_member(member: &XmlNode, index: &mut AppearanceIndex) {
    for appearance in child_elements(member) {
        if local_name(&appearance.name.0) == "Appearance" {
            index_appearance(appearance, index);
        }
    }
}

/// Index one `app:Appearance`: its theme's `app:ParameterizedTexture`s and
/// `app:X3DMaterial`s. The surface-data property is `surfaceData` in CityGML 3.0
/// and `surfaceDataMember` in 2.0.
fn index_appearance(appearance: &XmlNode, index: &mut AppearanceIndex) {
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
                "GeoreferencedTexture" => warn_unsupported(index, "GeoreferencedTexture"),
                // TODO: TexCoordGen (worldToTexture) as an alternative to explicit
                // textureCoordinates.
                _ => {}
            }
        }
    }
}

/// The side an `app:_SurfaceData` paints, from its `app:isFront` (default front).
fn side_of(surface_data: &XmlNode) -> Side {
    match child_text(surface_data, "isFront").as_deref() {
        Some("false") | Some("0") => Side::Back,
        _ => Side::Front,
    }
}

/// Merge `contribution` into one side of `surface` under `theme`, keeping one
/// material per (theme, side) that accumulates the colour and the texture bound to
/// it. The first colour and the first texture win, independent of document order;
/// a surface carrying both keeps both, so its texture is modulated by its colour.
fn set_side(
    index: &mut AppearanceIndex,
    surface: SurfaceKey,
    theme: &str,
    side: Side,
    contribution: Contribution,
) {
    let styles = index.surfaces.entry(surface).or_default();
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
    let sm = slot.get_or_insert_with(SideMaterial::default);
    match contribution {
        Contribution::Color(phong) => {
            if sm.color.is_none() {
                sm.color = Some(phong);
            }
        }
        Contribution::Texture(raster, sampler) => {
            if sm.texture.is_none() {
                sm.texture = Some((raster, sampler));
            }
        }
    }
}

/// Index one `app:ParameterizedTexture`: its image as a textured material bound to
/// each target surface's side, and each target ring's texture coordinates. Handles
/// both the CityGML 2.0 shape (`target` with a `uri` attribute, `textureCoordinates`
/// with a `ring` attribute) and the 3.0 shape (`textureParameterization` of
/// `TextureAssociation`, with element-text `target` / `ring`).
fn index_texture(texture: &XmlNode, theme: &str, index: &mut AppearanceIndex) {
    let Some(image) = child_text(texture, "imageURI") else {
        return;
    };
    let Some(uri) = resolve_uri(&texture.source_url, &image) else {
        return;
    };
    warn_unsupported_texture_fields(texture, index);
    let side = side_of(texture);
    let sampler = sampler_of(texture);
    let raster = Arc::new(Raster::Uri(uri));
    for child in child_elements(texture) {
        match local_name(&child.name.0) {
            "target" => {
                if let Some(target) = inline_target(child) {
                    index.add_texture_target(target, theme, side, &raster, sampler);
                }
            }
            "textureParameterization" => {
                for assoc in child_elements(child) {
                    if local_name(&assoc.name.0) != "TextureAssociation" {
                        continue;
                    }
                    if let Some(target) = texture_association(assoc) {
                        index.add_texture_target(target, theme, side, &raster, sampler);
                    }
                }
            }
            _ => {}
        }
    }
}

/// The [`Sampler`] for an `app:ParameterizedTexture`, from its `app:wrapMode`
/// mapped onto [`WrapMode`] for both axes. An absent or unrecognized mode keeps the
/// default repeat wrapping. CityGML has no per-axis wrap, so both axes share it.
fn sampler_of(texture: &XmlNode) -> Sampler {
    let Some(mode) = child_text(texture, "wrapMode") else {
        return Sampler::default();
    };
    let wrap = match mode.as_str() {
        "none" => WrapMode::None,
        "wrap" => WrapMode::Repeat,
        "mirror" => WrapMode::MirroredRepeat,
        "clamp" => WrapMode::ClampToEdge,
        "border" => WrapMode::ClampToBorder,
        other => {
            tracing::warn!("citygml appearance: unknown wrapMode {other:?}, using repeat");
            return Sampler::default();
        }
    };
    Sampler {
        wrap_s: wrap,
        wrap_t: wrap,
        ..Sampler::default()
    }
}

/// Warn once for each `app:ParameterizedTexture` sub-element this reader does not
/// model (`mimeType`, `borderColor`, `textureType`), so nothing is dropped silently.
fn warn_unsupported_texture_fields(texture: &XmlNode, index: &mut AppearanceIndex) {
    for field in ["mimeType", "borderColor", "textureType"] {
        if child_elements(texture).any(|e| local_name(&e.name.0) == field) {
            warn_unsupported(index, field);
        }
    }
}

/// Warn once per parser run that `feature`, an appearance sub-element this reader
/// does not model, was found and ignored.
fn warn_unsupported(index: &mut AppearanceIndex, feature: &'static str) {
    if index.warned.insert(feature) {
        tracing::warn!("citygml appearance: {feature} is not supported and was ignored");
    }
}

/// A white Phong material with no maps, the base for a texture bound to a surface
/// that carries no X3DMaterial colour.
fn white_phong() -> PhongMaterial {
    PhongMaterial {
        diffuse: [1.0, 1.0, 1.0],
        specular: [0.0; 3],
        emissive: [0.0; 3],
        ambient_intensity: 0.0,
        shininess: 0.0,
        transparency: 0.0,
        diffuse_map: None,
        emissive_map: None,
        normal_map: None,
    }
}

/// A texture bound to one surface: the surface's file-qualified id and each of its
/// rings' file-qualified id with its texture coordinates. The two CityGML encodings
/// are normalized to this shape so binding the material and recording the UV happen
/// through one path.
struct TextureTarget {
    surface: SurfaceKey,
    rings: Vec<(SurfaceKey, Vec<[f64; 2]>)>,
}

/// CityGML 2.0 texture target: `<app:target uri="#surface">` wrapping a
/// `TexCoordList` of `<app:textureCoordinates ring="#ring">`, where surface and
/// ring are `xlink` attributes resolved against the target element's own file.
fn inline_target(target: &XmlNode) -> Option<TextureTarget> {
    let surface = target_key(attr(target, "uri")?, &target.source_url)?;
    let mut rings = Vec::new();
    for coord_list in child_elements(target) {
        if local_name(&coord_list.name.0) != "TexCoordList" {
            continue;
        }
        for tc in child_elements(coord_list) {
            if local_name(&tc.name.0) != "textureCoordinates" {
                continue;
            }
            let Some(ring) = attr(tc, "ring").and_then(|r| target_key(r, &tc.source_url)) else {
                continue;
            };
            if let Some(uv) = parse_uv(&text_of(tc)) {
                rings.push((ring, uv));
            }
        }
    }
    Some(TextureTarget { surface, rings })
}

/// CityGML 3.0 texture target: an `<app:TextureAssociation>` with an element-text
/// `<app:target>#surface` and a nested `textureParameterization` / `TexCoordList`
/// whose `textureCoordinates` and `ring` are positional siblings.
fn texture_association(assoc: &XmlNode) -> Option<TextureTarget> {
    let surface = target_key(&child_text(assoc, "target")?, &assoc.source_url)?;
    let mut rings = Vec::new();
    for param in child_elements(assoc) {
        if local_name(&param.name.0) != "textureParameterization" {
            continue;
        }
        for coord_list in child_elements(param) {
            if local_name(&coord_list.name.0) == "TexCoordList" {
                rings.extend(tex_coord_pairs(coord_list));
            }
        }
    }
    Some(TextureTarget { surface, rings })
}

/// Pair a CityGML 3.0 `TexCoordList`'s `textureCoordinates` with its `ring`s by
/// document order: the i-th ring gets the i-th coordinate list. An unparseable
/// coordinate list keeps its slot (empty) so later pairs stay aligned; it is
/// dropped later when the UV length fails to match the ring. A list whose ring and
/// coordinate counts differ cannot be aligned positionally, so it is skipped whole.
fn tex_coord_pairs(coord_list: &XmlNode) -> Vec<(SurfaceKey, Vec<[f64; 2]>)> {
    let mut coords: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut rings: Vec<SurfaceKey> = Vec::new();
    for child in child_elements(coord_list) {
        match local_name(&child.name.0) {
            "textureCoordinates" => coords.push(parse_uv(&text_of(child)).unwrap_or_default()),
            "ring" => {
                if let Some(key) = target_key(&text_of(child), &child.source_url) {
                    rings.push(key);
                }
            }
            _ => {}
        }
    }
    if rings.len() != coords.len() {
        tracing::warn!(
            rings = rings.len(),
            coords = coords.len(),
            "citygml appearance: TexCoordList ring / textureCoordinates counts differ, skipped"
        );
        return Vec::new();
    }
    rings.into_iter().zip(coords).collect()
}

/// Index one `app:X3DMaterial`: a colour-only Phong material bound to each target
/// surface. Its `app:target`s are element text (`#surface-id`), not the `uri`
/// attribute a `ParameterizedTexture` uses.
fn index_material(material_node: &XmlNode, theme: &str, index: &mut AppearanceIndex) {
    if child_elements(material_node).any(|e| local_name(&e.name.0) == "isSmooth") {
        warn_unsupported(index, "X3DMaterial isSmooth");
    }
    let phong = PhongMaterial {
        diffuse: child_color(material_node, "diffuseColor", [0.8, 0.8, 0.8]),
        specular: child_color(material_node, "specularColor", [1.0, 1.0, 1.0]),
        emissive: child_color(material_node, "emissiveColor", [0.0, 0.0, 0.0]),
        ambient_intensity: child_f32(material_node, "ambientIntensity", 0.2),
        shininess: child_f32(material_node, "shininess", 0.2),
        transparency: child_f32(material_node, "transparency", 0.0),
        diffuse_map: None,
        emissive_map: None,
        normal_map: None,
    };
    let side = side_of(material_node);
    for target in child_elements(material_node) {
        if local_name(&target.name.0) != "target" {
            continue;
        }
        let Some(surface) = target_key(&text_of(target), &target.source_url) else {
            continue;
        };
        set_side(
            index,
            surface,
            theme,
            side,
            Contribution::Color(phong.clone()),
        );
    }
}

/// A material colour element (three floats), or `default` when absent or malformed.
fn child_color(node: &XmlNode, name: &str, default: [f32; 3]) -> [f32; 3] {
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
fn child_f32(node: &XmlNode, name: &str, default: f32) -> f32 {
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
fn child_elements(node: &XmlNode) -> impl Iterator<Item = &XmlNode> {
    node.children.iter().filter_map(|c| match c {
        XmlChild::Element(e) => Some(e.as_ref()),
        _ => None,
    })
}

/// The trimmed text of the first child element named `name`, if non-empty.
fn child_text(node: &XmlNode, name: &str) -> Option<String> {
    child_elements(node)
        .find(|e| local_name(&e.name.0) == name)
        .map(|e| text_of(e).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// A node's concatenated text content.
fn text_of(node: &XmlNode) -> String {
    node.children
        .iter()
        .filter_map(|c| match c {
            XmlChild::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

/// The value of the attribute named `name`, ignoring namespace.
fn attr<'a>(node: &'a XmlNode, name: &str) -> Option<&'a str> {
    node.attrs
        .iter()
        .find(|((q, _), _)| local_name(q) == name)
        .map(|(_, v)| v.as_str())
}

/// Resolve an appearance target / ring reference to a file-qualified id. A local
/// `#id` scopes to `base` (the file the reference is written in); an explicit
/// `file#id` joins `file` against `base`, mirroring a geometry `xlink:href`; a bare
/// `id` with no `#` is treated as a local reference. `None` for an empty reference
/// or an unjoinable file.
fn target_key(reference: &str, base: &Url) -> Option<SurfaceKey> {
    let reference = reference.trim();
    if let Some(frag) = reference.strip_prefix('#') {
        Some((base.as_str().to_string(), frag.to_string()))
    } else if let Some((file, frag)) = reference.split_once('#') {
        base.join(file)
            .ok()
            .map(|u| (u.to_string(), frag.to_string()))
    } else if reference.is_empty() {
        None
    } else {
        Some((base.as_str().to_string(), reference.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::citygml_parser::parser::Parser;
    use crate::citygml_parser::resolver::resolve_root;
    use reearth_flow_geometry::appearance::{
        Appearance, Material, Sampler, Side, ThemeId, UvSet, UvSource, WrapMode,
    };
    use reearth_flow_geometry::Euclidean3DGeometry;
    use std::sync::Arc;
    use url::Url;

    /// The UV sets carried by a geometry leaf's appearance (they now live inside the
    /// appearance, per theme), or empty when the leaf carries no appearance.
    fn uv_sets(appearance: &Option<Appearance>) -> Vec<&UvSet> {
        appearance
            .as_ref()
            .map(|a| a.uv_iter().collect())
            .unwrap_or_default()
    }

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
        let (pending, raw_registry, geom_registry, appearance_members, _ns) = parser.finish();
        let appearance = super::build_index(&appearance_members, &raw_registry);
        assert!(!appearance.is_empty(), "appearance should be indexed");
        let feature = pending.into_iter().next().expect("one feature");
        let geom = resolve_root(&feature.geoms[0].node, &geom_registry, &appearance)
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
        assert_eq!(
            appearance.default_theme(),
            &ThemeId(Arc::from("rgbTexture"))
        );
        assert!(matches!(
            &appearance.materials()[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        let uv = uv_sets(polygon.appearance());
        assert_eq!(uv.len(), 1);
        let UvSource::Explicit(coords) = &uv[0].uv else {
            panic!("expected explicit UV");
        };
        // The ring's four (closed) vertices each carry a UV pair.
        assert_eq!(coords.len(), 4);
        assert_eq!(coords[1], [1.0, 0.0]);
    }

    /// The sampler of the polygon's diffuse texture when its `ParameterizedTexture`
    /// declares the given `app:wrapMode`.
    fn sampler_for_wrap_mode(mode: &str) -> Sampler {
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
                  <app:wrapMode>{mode}</app:wrapMode>
                  <app:target uri="#poly1"><app:TexCoordList>
                    <app:textureCoordinates ring="#ring1">0 0 1 0 1 1 0 0</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceDataMember>
              </app:Appearance></app:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let Material::Phong(m) = &polygon.appearance().as_ref().expect("textured").materials()[0]
        else {
            panic!("expected Phong");
        };
        m.diffuse_map.as_ref().expect("has diffuse map").sampler
    }

    #[test]
    fn wrap_mode_maps_onto_sampler() {
        // Each CityGML wrapMode maps to the matching WrapMode on both axes.
        assert_eq!(sampler_for_wrap_mode("none").wrap_s, WrapMode::None);
        assert_eq!(sampler_for_wrap_mode("wrap").wrap_s, WrapMode::Repeat);
        assert_eq!(
            sampler_for_wrap_mode("mirror").wrap_s,
            WrapMode::MirroredRepeat
        );
        assert_eq!(sampler_for_wrap_mode("clamp").wrap_s, WrapMode::ClampToEdge);
        let border = sampler_for_wrap_mode("border");
        assert_eq!(border.wrap_s, WrapMode::ClampToBorder);
        assert_eq!(border.wrap_t, WrapMode::ClampToBorder, "both axes share it");
    }

    #[test]
    fn absent_wrap_mode_keeps_default_sampler() {
        // A texture with no wrapMode keeps the default (repeat) sampler.
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
        let Material::Phong(m) = &polygon.appearance().as_ref().expect("textured").materials()[0]
        else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse_map.as_ref().unwrap().sampler, Sampler::default());
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
        assert!(uv_sets(polygon.appearance()).is_empty());
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
        let Material::Phong(m) = &appearance.materials()[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.588235, 0.588235, 0.588235]);
        assert!(m.diffuse_map.is_none(), "colour-only, no texture");
        assert!(
            uv_sets(polygon.appearance()).is_empty(),
            "colour-only, no UV"
        );
    }

    #[test]
    fn texture_and_colour_merge_on_same_surface() {
        // An X3DMaterial and a ParameterizedTexture on the same surface + theme merge
        // into one material: the colour is the base and the texture its diffuse map,
        // independent of document order (here the colour is listed first).
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
        assert_eq!(appearance.materials().len(), 1, "one merged material");
        let Material::Phong(m) = &appearance.materials()[0] else {
            panic!("expected Phong");
        };
        assert!(m.diffuse_map.is_some(), "the texture is the diffuse map");
        assert_eq!(
            m.diffuse,
            [0.1, 0.2, 0.3],
            "the X3DMaterial colour is the base"
        );
        assert_eq!(
            uv_sets(polygon.appearance()).len(),
            1,
            "the texture's UV is present"
        );
    }

    #[test]
    fn xlink_surface_data_is_resolved() {
        // themeB's surface data is an `xlink:href` to the X3DMaterial defined inline
        // under themeA. Resolving the reference binds poly1 under themeB too; without
        // it, themeB would carry no style.
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
                <app:theme>themeA</app:theme>
                <app:surfaceData><app:X3DMaterial gml:id="mat1">
                  <app:diffuseColor>0.2 0.4 0.6</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>themeB</app:theme>
                <app:surfaceData xlink:href="#mat1"/>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon styled");
        let themes: Vec<&str> = appearance.themes().iter().map(|t| &*t.theme.0).collect();
        assert!(themes.contains(&"themeA"), "inline surface data bound");
        assert!(
            themes.contains(&"themeB"),
            "xlink-referenced surface data bound under its own theme"
        );
    }

    #[test]
    fn xlink_appearance_member_is_resolved() {
        // A self-closing `app:appearanceMember` references a whole `app:Appearance`
        // by `xlink:href`; the shared appearance's material reaches poly1.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance gml:id="appA">
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0.7 0.7 0.7</app:diffuseColor>
                  <app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
              <core:appearanceMember xlink:href="#appA"/>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon styled");
        assert_eq!(appearance.themes().len(), 1, "one theme, not duplicated");
        let Material::Phong(m) = &appearance.materials()[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.7, 0.7, 0.7]);
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
        let Material::Phong(m) = &appearance.materials()[0] else {
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
        assert_eq!(appearance.materials().len(), 2, "front and back materials");
        assert!(
            appearance.themes()[0].back.is_some(),
            "the back side is bound"
        );
        let colours: Vec<[f32; 3]> = appearance
            .materials()
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
        let uv = uv_sets(polygon.appearance());
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
        let Material::Phong(m) = &appearance.materials()[0] else {
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
            appearance.materials().len(),
            1,
            "only the leaf material attaches"
        );
        let Material::Phong(m) = &appearance.materials()[0] else {
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
            &appearance.materials()[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        let uv = uv_sets(polygon.appearance());
        assert_eq!(uv.len(), 1);
        let UvSource::Explicit(coords) = &uv[0].uv else {
            panic!("expected explicit UV");
        };
        assert_eq!(coords.len(), 4);
        assert_eq!(coords[1], [1.0, 0.0]);
    }

    /// The `TriangularMesh` of a feature whose sole geometry is a
    /// `TriangulatedSurface` leaf (resolved directly, not wrapped in a collection).
    fn resolve_only_mesh(xml: &str) -> reearth_flow_geometry::triangular_mesh::TriangularMesh3D {
        let mut parser = Parser::new();
        parser
            .parse(xml.as_bytes(), &Url::parse("file:///dir/test.gml").unwrap())
            .unwrap();
        let (pending, raw_registry, geom_registry, appearance_members, _ns) = parser.finish();
        let appearance = super::build_index(&appearance_members, &raw_registry);
        let feature = pending.into_iter().next().expect("one feature");
        let geom = resolve_root(&feature.geoms[0].node, &geom_registry, &appearance)
            .expect("geometry resolves");
        match geom {
            Euclidean3DGeometry::TriangularMesh(m) => *m,
            other => panic!("expected TriangularMesh, got {other:?}"),
        }
    }

    /// A `TriangulatedSurface` whose texture targets the surface, with per-triangle
    /// texture coordinates keyed by each triangle's `LinearRing`. Each triangle's
    /// three corners get a UV, giving `3 * triangle_count` for the welded mesh.
    #[test]
    fn parameterized_texture_attaches_to_triangular_mesh() {
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:TriangulatedSurface gml:id="ts1">
                  <gml:trianglePatches>
                    <gml:Triangle><gml:exterior><gml:LinearRing gml:id="tri1">
                      <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                    </gml:LinearRing></gml:exterior></gml:Triangle>
                    <gml:Triangle><gml:exterior><gml:LinearRing gml:id="tri2">
                      <gml:posList>1 0 0 1 1 0 0 1 0 1 0 0</gml:posList>
                    </gml:LinearRing></gml:exterior></gml:Triangle>
                  </gml:trianglePatches>
                </gml:TriangulatedSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:ParameterizedTexture>
                  <app:imageURI>tex/tin.jpg</app:imageURI>
                  <app:textureParameterization><app:TextureAssociation>
                    <app:target>#ts1</app:target>
                    <app:textureParameterization><app:TexCoordList>
                      <app:textureCoordinates>0 0 1 0 0 1 0 0</app:textureCoordinates>
                      <app:ring>#tri1</app:ring>
                    </app:TexCoordList></app:textureParameterization>
                  </app:TextureAssociation></app:textureParameterization>
                  <app:textureParameterization><app:TextureAssociation>
                    <app:target>#ts1</app:target>
                    <app:textureParameterization><app:TexCoordList>
                      <app:textureCoordinates>1 0 1 1 0 1 1 0</app:textureCoordinates>
                      <app:ring>#tri2</app:ring>
                    </app:TexCoordList></app:textureParameterization>
                  </app:TextureAssociation></app:textureParameterization>
                </app:ParameterizedTexture></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let mesh = resolve_only_mesh(&xml);
        assert_eq!(mesh.num_triangles(), 2);
        let appearance = mesh.appearance().as_ref().expect("mesh is textured");
        assert!(matches!(
            &appearance.materials()[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        let uv = uv_sets(mesh.appearance());
        assert_eq!(uv.len(), 1);
        assert_eq!(uv[0].side, Side::Front);
        let UvSource::Explicit(coords) = &uv[0].uv else {
            panic!("expected explicit UV");
        };
        // One UV per triangle corner: the two triangles' first three coordinates.
        assert_eq!(coords.len(), 6);
        assert_eq!(coords[3], [1.0, 0.0], "second triangle's first corner");
    }

    /// A texture targeting the surface but referencing a ring id no triangle carries
    /// cannot assemble full UV, so the mesh is left bare rather than mismatched.
    #[test]
    fn triangular_mesh_missing_ring_uv_leaves_mesh_bare() {
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:TriangulatedSurface gml:id="ts1">
                  <gml:trianglePatches>
                    <gml:Triangle><gml:exterior><gml:LinearRing gml:id="tri1">
                      <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                    </gml:LinearRing></gml:exterior></gml:Triangle>
                  </gml:trianglePatches>
                </gml:TriangulatedSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:ParameterizedTexture>
                  <app:imageURI>tex/tin.jpg</app:imageURI>
                  <app:textureParameterization><app:TextureAssociation>
                    <app:target>#ts1</app:target>
                    <app:textureParameterization><app:TexCoordList>
                      <app:textureCoordinates>0 0 1 0 0 1 0 0</app:textureCoordinates>
                      <app:ring>#other</app:ring>
                    </app:TexCoordList></app:textureParameterization>
                  </app:TextureAssociation></app:textureParameterization>
                </app:ParameterizedTexture></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let mesh = resolve_only_mesh(&xml);
        assert!(mesh.appearance().is_none());
        assert!(uv_sets(mesh.appearance()).is_empty());
    }

    /// An `X3DMaterial` colour targeting the surface attaches to the mesh with no UV.
    #[test]
    fn x3d_material_colour_attaches_to_triangular_mesh() {
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:TriangulatedSurface gml:id="ts1">
                  <gml:trianglePatches>
                    <gml:Triangle><gml:exterior><gml:LinearRing gml:id="tri1">
                      <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                    </gml:LinearRing></gml:exterior></gml:Triangle>
                  </gml:trianglePatches>
                </gml:TriangulatedSurface></bldg:lod2MultiSurface>
              </bldg:Building></core:cityObjectMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0.2 0.4 0.6</app:diffuseColor>
                  <app:target>#ts1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let mesh = resolve_only_mesh(&xml);
        let appearance = mesh.appearance().as_ref().expect("mesh is coloured");
        let Material::Phong(m) = &appearance.materials()[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.2, 0.4, 0.6]);
        assert!(m.diffuse_map.is_none(), "colour-only, no texture");
        assert!(uv_sets(mesh.appearance()).is_empty(), "colour-only, no UV");
    }

    #[test]
    fn container_theme_inherited_alongside_leaf_theme() {
        // The leaf polygon carries a style under `shared` (red) and the enclosing
        // MultiSurface carries one under `shared` (green) and one under `only`
        // (blue). The leaf wins for `shared`, and `only` is inherited from the
        // container: the polygon ends up with both themes, `shared` red.
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
                <app:theme>shared</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>1 0 0</app:diffuseColor><app:target>#poly1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>shared</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0 1 0</app:diffuseColor><app:target>#ms1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
              <core:appearanceMember><app:Appearance>
                <app:theme>only</app:theme>
                <app:surfaceData><app:X3DMaterial>
                  <app:diffuseColor>0 0 1</app:diffuseColor><app:target>#ms1</app:target>
                </app:X3DMaterial></app:surfaceData>
              </app:Appearance></core:appearanceMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon.appearance().as_ref().expect("polygon styled");
        let themes: Vec<&str> = appearance.themes().iter().map(|t| &*t.theme.0).collect();
        assert!(themes.contains(&"shared"), "leaf theme present");
        assert!(themes.contains(&"only"), "container-only theme inherited");
        // The leaf's `shared` (red) wins over the container's `shared` (green).
        let reds = appearance
            .materials()
            .iter()
            .filter(|m| matches!(m, Material::Phong(p) if p.diffuse == [1.0, 0.0, 0.0]))
            .count();
        let greens = appearance
            .materials()
            .iter()
            .filter(|m| matches!(m, Material::Phong(p) if p.diffuse == [0.0, 1.0, 0.0]))
            .count();
        assert_eq!(reds, 1, "leaf red kept");
        assert_eq!(
            greens, 0,
            "container green suppressed by leaf for the same theme"
        );
    }

    #[test]
    fn nested_appearance_property_is_indexed() {
        // An appearance declared as an `app:appearance` property on the Building
        // (not a top-level `appearanceMember`) still reaches its target surface.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:Polygon>
                </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
                <app:appearance><app:Appearance>
                  <app:theme>rgbTexture</app:theme>
                  <app:surfaceData><app:X3DMaterial>
                    <app:diffuseColor>0.3 0.5 0.7</app:diffuseColor>
                    <app:target>#poly1</app:target>
                  </app:X3DMaterial></app:surfaceData>
                </app:Appearance></app:appearance>
              </bldg:Building></core:cityObjectMember>
            </core:CityModel>"##
        );
        let Polygon3DOut(polygon) = resolve_only_polygon(&xml);
        let appearance = polygon
            .appearance()
            .as_ref()
            .expect("nested appearance reaches the surface");
        let Material::Phong(m) = &appearance.materials()[0] else {
            panic!("expected Phong");
        };
        assert_eq!(m.diffuse, [0.3, 0.5, 0.7]);
    }

    /// The welded `PolygonMesh` of a feature whose sole geometry is an inline
    /// `Surface` of polygon patches.
    fn resolve_only_polygon_mesh(xml: &str) -> reearth_flow_geometry::polygon_mesh::PolygonMesh3D {
        let mut parser = Parser::new();
        parser
            .parse(xml.as_bytes(), &Url::parse("file:///dir/test.gml").unwrap())
            .unwrap();
        let (pending, raw_registry, geom_registry, appearance_members, _ns) = parser.finish();
        let appearance = super::build_index(&appearance_members, &raw_registry);
        let feature = pending.into_iter().next().expect("one feature");
        let geom = resolve_root(&feature.geoms[0].node, &geom_registry, &appearance)
            .expect("geometry resolves");
        match geom {
            Euclidean3DGeometry::PolygonMesh(m) => *m,
            other => panic!("expected PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn parameterized_texture_attaches_to_inline_surface() {
        // An inline gml:Surface of two PolygonPatches welds into a PolygonMesh; a
        // texture targeting the surface, with per-ring UV, binds each patch by its
        // ring and is carried across the weld into the mesh.
        let xml = format!(
            r##"<core:CityModel {NS}>
              <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:lod2Surface><gml:Surface gml:id="surf1"><gml:patches>
                  <gml:PolygonPatch><gml:exterior><gml:LinearRing gml:id="ringA">
                    <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:PolygonPatch>
                  <gml:PolygonPatch><gml:exterior><gml:LinearRing gml:id="ringB">
                    <gml:posList>1 0 0 2 0 0 1 1 0 1 0 0</gml:posList>
                  </gml:LinearRing></gml:exterior></gml:PolygonPatch>
                </gml:patches></gml:Surface></bldg:lod2Surface>
              </bldg:Building></core:cityObjectMember>
              <app:appearanceMember><app:Appearance>
                <app:theme>rgbTexture</app:theme>
                <app:surfaceDataMember><app:ParameterizedTexture>
                  <app:imageURI>tex/a.jpg</app:imageURI>
                  <app:target uri="#surf1"><app:TexCoordList>
                    <app:textureCoordinates ring="#ringA">0 0 1 0 1 1 0 0</app:textureCoordinates>
                    <app:textureCoordinates ring="#ringB">0 0 1 0 1 1 0 0</app:textureCoordinates>
                  </app:TexCoordList></app:target>
                </app:ParameterizedTexture></app:surfaceDataMember>
              </app:Appearance></app:appearanceMember>
            </core:CityModel>"##
        );
        let mesh = resolve_only_polygon_mesh(&xml);
        assert_eq!(mesh.num_faces(), 2);
        let appearance = mesh.appearance().as_ref().expect("mesh is textured");
        assert!(matches!(
            &appearance.materials()[0],
            Material::Phong(m) if m.diffuse_map.is_some()
        ));
        assert_eq!(uv_sets(mesh.appearance()).len(), 1, "one welded UV set");
    }

    #[test]
    fn same_gml_id_in_two_files_do_not_cross_bind() {
        // Both files define gml:id="poly1" with a different X3DMaterial colour.
        // Keying appearance by (file, gml:id) binds each file's material only to
        // that file's polygon; a bare-id index would let one file's colour leak
        // onto the other's surface.
        fn doc(color: &str) -> String {
            format!(
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
                      <app:diffuseColor>{color}</app:diffuseColor>
                      <app:target>#poly1</app:target>
                    </app:X3DMaterial></app:surfaceData>
                  </app:Appearance></core:appearanceMember>
                </core:CityModel>"##
            )
        }
        fn diffuse(geom: &Euclidean3DGeometry) -> [f32; 3] {
            let Euclidean3DGeometry::Collection(c) = geom else {
                panic!("expected collection");
            };
            let Euclidean3DGeometry::Polygon(p) = c.members().first().expect("one member") else {
                panic!("expected polygon");
            };
            let Material::Phong(m) = &p.appearance().as_ref().expect("styled").materials()[0]
            else {
                panic!("expected phong");
            };
            m.diffuse
        }
        let mut parser = Parser::new();
        parser
            .parse(
                doc("1 0 0").as_bytes(),
                &Url::parse("file:///dir/a.gml").unwrap(),
            )
            .unwrap();
        parser
            .parse(
                doc("0 0 1").as_bytes(),
                &Url::parse("file:///dir/b.gml").unwrap(),
            )
            .unwrap();
        let (pending, raw_registry, geom_registry, appearance_members, _ns) = parser.finish();
        let appearance = super::build_index(&appearance_members, &raw_registry);
        let features: Vec<_> = pending.into_iter().collect();
        assert_eq!(features.len(), 2);
        let a = resolve_root(&features[0].geoms[0].node, &geom_registry, &appearance)
            .expect("file a resolves");
        let b = resolve_root(&features[1].geoms[0].node, &geom_registry, &appearance)
            .expect("file b resolves");
        assert_eq!(diffuse(&a), [1.0, 0.0, 0.0], "file a keeps its own colour");
        assert_eq!(diffuse(&b), [0.0, 0.0, 1.0], "file b keeps its own colour");
    }
}
