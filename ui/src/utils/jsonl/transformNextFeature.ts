/**
 * Transform for the engine's new geometry model.
 *
 * The shape differs from the legacy one in three ways that matter here: the
 * geometry is an externally-tagged enum, so its type is a JSON key rather than
 * something to infer from the payload; the CRS moved from one `epsg` per
 * feature onto a `frame` per coordinate-bearing leaf; and coordinates are
 * `[x, y]` arrays instead of `{ x, y }` objects.
 *
 * Geometry becomes GeoJSON in both embedding dimensions — a GeoJSON position
 * takes an optional third element, so a 3D point needs no mesh pipeline to
 * draw. Only a point cloud and a CSG tree get a descriptive stand-in instead:
 * a cloud would be millions of positions, and a boolean tree is unevaluated.
 *
 * A whole `Geometry` is converted, not just a leaf, because a CityGML feature
 * arrives as a `GeometryCollection` of per-LOD members and judging it by its
 * own kind would conclude there is nothing to draw.
 */
import i18n from "@flow/lib/i18n/i18n";
import {
  describeGeometry,
  type GeometryDescription,
} from "@flow/lib/intermediateData";

type Position = number[];

/**
 * The finest level of detail a CityGML feature carries, kept for the map to
 * swap in when the feature is selected.
 *
 * The map draws one level — see {@link LOD_PREFERENCE} — because stacking a
 * coarse box inside a detailed one draws neither well. Legacy did the same and
 * upgraded the selected feature to its finest level
 * (`useLodWorker.prepareWorkerInput`: LOD3, else LOD2), which it could do
 * because it held the engine's whole record. This holds the one extra level
 * that upgrade needs, and nothing else.
 *
 * Two levels rather than all of them is the difference between ~154 MB and
 * ~870 MB over the panel's 2000-feature limit, on CityGML shaped like PLATEAU;
 * `DISPLAY_POSITION_LIMIT` in `useStreamingDebugRunQuery` counts this too, so a
 * file dense enough for even two levels to matter yields fewer features rather
 * than a dead tab.
 */
export type LodDetail = {
  lod: number;
  geometry: Record<string, unknown>;
};

export type TransformedFeature = {
  id: string;
  type: "Feature";
  properties: Record<string, unknown>;
  geometry?: unknown;
  /**
   * Not part of the drawn geometry, and deliberately not on it: the table
   * builds a column per geometry key, and a second coordinate blob there is
   * noise. Only `useLodWorker` reads it.
   */
  lodDetail?: LodDetail;
};

/**
 * What the derived form drops, for whoever needs it back.
 *
 * This carries what the map draws from: GeoJSON coordinates, the frame, the
 * chosen level of detail and the one finer level behind it, and appearance
 * flattened to one colour per surface. The engine's record holds more — the
 * levels between those two, every theme rather than the default one, textures,
 * UV sets, a tangent frame's basis — and a geometry bug is often in one of
 * those.
 *
 * None of it is retained, because keeping every parsed record alongside every
 * derived one cost ~3.5x memory (~35 MB to ~123 MB over the panel's
 * 2000-feature `displayLimit`, shaped like PLATEAU CityGML). If it is wanted
 * again, fetch the one selected feature's line on demand rather than retaining
 * all of them.
 */

/** Human-readable coordinate frame, since the EPSG is no longer feature-level. */
function describeFrame(frame: unknown): string | undefined {
  if (frame === "Euclidean") return "Euclidean";
  if (!frame || typeof frame !== "object") return undefined;

  const record = frame as Record<string, unknown>;
  if (typeof record.Crs === "number") return `EPSG:${record.Crs}`;
  if (record.Tangent && typeof record.Tangent === "object") {
    const base = (record.Tangent as Record<string, unknown>).base;
    const anchor = describeFrame(base);
    return anchor ? `Tangent plane (${anchor})` : "Tangent plane";
  }
  return undefined;
}

/**
 * Whether a CRS declares its axes north/latitude first.
 *
 * The engine stores coordinates "in the order declared by the CRS authority"
 * (see `engine/runtime/geometry/src/coordinate.rs`), which for geographic CRSs
 * and for the Japan Plane Rectangular systems is northing first. GeoJSON is
 * always `[longitude, latitude]`, so those need swapping on the way out.
 *
 * The engine resolves this through PROJ (`ops/reproject/ffi.rs`,
 * `axis_order_sign`), which the browser has no equivalent of. So this mirrors
 * the CRSs the engine actually supports rather than trying to be general: the
 * authoritative list is the 75-entry `WKT1_ESRI` table in
 * `engine/runtime/action-sink/src/file/shapefile/crs.rs`, and every code in it
 * is covered below or is deliberately east-first (EPSG:3857 Web Mercator).
 *
 * This is not a Japan-only restriction on principle — it is the engine's
 * current reach. When the engine gains a CRS, add it here: anything unlisted
 * passes through east-first, which is right for Web Mercator and UTM but would
 * silently transpose a north-first system.
 */
function isNorthFirst(epsg: number): boolean {
  // Geographic CRSs: WGS 84, JGD2000 and JGD2011, with their 3D forms.
  // EPSG:6697 (JGD2011 + height) is what PLATEAU CityGML carries.
  if ([4326, 4979, 4612, 6667, 6668, 6697].includes(epsg)) return true;
  // Japan Plane Rectangular CS, whose axes are (X = north, Y = east).
  if (epsg >= 2443 && epsg <= 2461) return true; // JGD2000, zones I-XIX
  if (epsg >= 6669 && epsg <= 6687) return true; // JGD2011, zones I-XIX
  if (epsg >= 30161 && epsg <= 30179) return true; // Tokyo datum, zones I-XIX
  // Same, compounded with JGD2011 vertical height. Only zones I-XIII exist.
  if (epsg >= 10162 && epsg <= 10174) return true;
  return false;
}

/**
 * Whether a leaf's coordinates need swapping to reach GeoJSON order. Only a
 * CRS frame can: a tangent plane stores in-plane `(x, y)` metres, and a bare
 * Euclidean frame is `(x, y)` by definition.
 */
function needsAxisSwap(frame: unknown): boolean {
  if (!frame || typeof frame !== "object") return false;
  const crs = (frame as Record<string, unknown>).Crs;
  return typeof crs === "number" && isNorthFirst(crs);
}

/**
 * Put one coordinate into GeoJSON order, appending the leaf's single elevation
 * if it carries one. Every coordinate this module emits goes through here.
 *
 * When neither applies the original array is returned rather than copied —
 * an allocation per position, on files that are nothing but positions, for no
 * gain. The parsed record is discarded once the transform returns, so this
 * keeps alive only the coordinates themselves, which are the data being kept
 * either way; nothing downstream mutates them.
 */
function toPosition(coord: Position, swap: boolean, z: unknown): Position {
  if (!swap && typeof z !== "number") return coord;

  const ordered = swap ? [coord[1], coord[0], ...coord.slice(2)] : [...coord];
  if (typeof z === "number") ordered.push(z);
  return ordered;
}

function toPositions(
  coords: Position[],
  swap: boolean,
  z: unknown,
): Position[] {
  return coords.map((coord) => toPosition(coord, swap, z));
}

/** GeoJSON requires a closed ring; the engine's validity rules already expect one. */
function closeRing(ring: Position[]): Position[] {
  if (ring.length === 0) return ring;
  const [first] = ring;
  const last = ring[ring.length - 1];
  const closed = first.every((value, axis) => value === last[axis]);
  return closed ? ring : [...ring, first];
}

function toRing(coords: Position[], swap: boolean, z: unknown): Position[] {
  return closeRing(toPositions(coords, swap, z));
}

function ringsOfFace(face: unknown, swap: boolean, z: unknown): Position[][] {
  const record = (face ?? {}) as Record<string, unknown>;
  const exterior = (record.exterior ?? []) as Position[];
  const holes = (record.holes ?? []) as Position[][];
  return [
    toRing(exterior, swap, z),
    ...holes.map((hole) => toRing(hole, swap, z)),
  ];
}

/** Which embedding dimension a leaf came from, for recursing into collections. */
type Dimension = "Euclidean2D" | "Euclidean3D";

/**
 * Leaf variants with a GeoJSON form. A point cloud would be millions of
 * positions and a CSG tree is unevaluated, so those are described rather than
 * drawn — which is also what decides whether a file gets a map at all.
 */
const GEOJSON_VARIANTS = new Set([
  "Point",
  "LineString",
  "Polygon",
  "PolygonMesh",
  "TriangularMesh",
  "Solid",
  "Collection",
]);

export function hasGeoJsonForm(variant: string | null): boolean {
  return variant !== null && GEOJSON_VARIANTS.has(variant);
}

/**
 * One material in the form the CityGML renderer reads.
 *
 * `resolveAppearanceColor` in `cityGmlGeometryToPrimitives` takes a
 * `diffuseColor` triple and a `transparency`, which is what the legacy record
 * carried; the new appearance model is richer, so this is the lossy projection
 * onto what `PerInstanceColorAppearance` can actually draw. Textures are not
 * part of it — that appearance has no texture support, and legacy dropped them
 * for the same reason.
 */
type RendererMaterial = { diffuseColor: number[]; transparency: number };

/**
 * Surface-to-material binding for one converted geometry, under the field names
 * the renderer already reads.
 */
type Shading = {
  materials: (RendererMaterial | null)[];
  /** One entry per emitted surface, indexing `materials`; null = unbound. */
  polygonMaterials: (number | null)[];
};

/** Project a `Material` onto a flat colour. Null when it is neither shading model. */
function toRendererMaterial(material: unknown): RendererMaterial | null {
  const record = (material ?? {}) as Record<string, unknown>;

  const phong = record.Phong as Record<string, unknown> | undefined;
  if (phong) {
    return {
      diffuseColor: (phong.diffuse as number[]) ?? [1, 1, 1],
      transparency:
        typeof phong.transparency === "number" ? phong.transparency : 0,
    };
  }

  const pbr = record.Pbr as Record<string, unknown> | undefined;
  if (pbr) {
    // `base_color` carries alpha as its fourth element; the renderer wants the
    // inverse, as CityGML states it.
    const base = (pbr.base_color as number[]) ?? [1, 1, 1, 1];
    return {
      diffuseColor: base.slice(0, 3),
      transparency: 1 - (typeof base[3] === "number" ? base[3] : 1),
    };
  }

  return null;
}

/**
 * A leaf's appearance, flattened to one material index per surface.
 *
 * The engine binds faces to materials per theme, and a leaf always names a
 * default theme for "a single-theme consumer to render" — which is what the
 * map is. So this reads that theme's front binding and ignores the rest; the
 * unused themes, the back bindings and the UV sets are not something a flat
 * per-instance colour can express.
 */
function shadingOf(appearance: unknown, surfaces: number): Shading | null {
  const record = appearance as Record<string, unknown> | null | undefined;
  if (!record || !Array.isArray(record.materials)) return null;

  const themes = record.themes;
  if (!Array.isArray(themes) || themes.length === 0) return null;

  const theme =
    themes.find(
      (entry) =>
        (entry as Record<string, unknown>)?.theme === record.default_theme,
    ) ?? themes[0];
  const front = (theme as Record<string, unknown>)?.front as
    | Record<string, unknown>
    | undefined;
  if (!front) return null;

  let polygonMaterials: (number | null)[];
  if (typeof front.Uniform === "number") {
    polygonMaterials = new Array(surfaces).fill(front.Uniform);
  } else if (Array.isArray(front.PerFace)) {
    const perFace = front.PerFace as unknown[];
    polygonMaterials = Array.from({ length: surfaces }, (_, index) =>
      typeof perFace[index] === "number" ? (perFace[index] as number) : null,
    );
  } else {
    return null;
  }

  return {
    materials: record.materials.map(toRendererMaterial),
    polygonMaterials,
  };
}

/** Spread a leaf's shading onto its converted geometry, or nothing when it has none. */
function shadingFields(
  appearance: unknown,
  surfaces: number,
): Record<string, unknown> {
  const shading = shadingOf(appearance, surfaces);
  return shading ?? {};
}

/** Surfaces a converted geometry emitted, for lining bindings up with them. */
function surfaceCountOf(geometry: Record<string, unknown>): number {
  if (geometry.type === "MultiPolygon") {
    return Array.isArray(geometry.coordinates)
      ? geometry.coordinates.length
      : 0;
  }
  return geometry.type === "Polygon" ? 1 : 0;
}

/**
 * Concatenate the members' shading, rebasing each one's indices onto the
 * combined palette.
 *
 * A member with no appearance still has to advance the binding by its own
 * surface count, or every index after it points at the wrong material.
 */
function mergeShading(
  geometries: Record<string, unknown>[],
): Record<string, unknown> {
  if (
    !geometries.some((geometry) => Array.isArray(geometry.polygonMaterials))
  ) {
    return {};
  }

  const materials: unknown[] = [];
  const polygonMaterials: (number | null)[] = [];

  for (const geometry of geometries) {
    const own = geometry.polygonMaterials as (number | null)[] | undefined;
    const palette = geometry.materials as unknown[] | undefined;

    if (Array.isArray(own) && Array.isArray(palette)) {
      const offset = materials.length;
      materials.push(...palette);
      polygonMaterials.push(
        ...own.map((index) => (index === null ? null : index + offset)),
      );
    } else {
      for (let i = 0; i < surfaceCountOf(geometry); i++) {
        polygonMaterials.push(null);
      }
    }
  }

  return { materials, polygonMaterials };
}

/**
 * Convert a geometry leaf to GeoJSON.
 *
 * 2D and 3D leaves use the same field names and differ only in coordinate
 * arity, and a GeoJSON position takes an optional third element, so one
 * conversion serves both. Meshes and solids flatten to a MultiPolygon — one
 * polygon per face or triangle — which is what Cesium draws.
 *
 * A leaf that carries an appearance also emits `materials` and
 * `polygonMaterials`, the two fields the CityGML renderer colours from. Legacy
 * got those by passing the engine's record through untouched; the new model
 * keeps appearance per leaf, so it is projected here.
 */
function toGeoJson(
  variant: string | null,
  value: unknown,
  dimension: Dimension,
): Record<string, unknown> | null {
  if (!hasGeoJsonForm(variant)) return null;

  const leaf = (value ?? {}) as Record<string, unknown>;
  const z = leaf.z;
  // Read per leaf, not per feature: a collection's members may sit in
  // different frames, and only some of them may need swapping.
  const swap = needsAxisSwap(leaf.frame);

  switch (variant) {
    case "Point": {
      const position = leaf.position as Position | undefined;
      return position
        ? { type: "Point", coordinates: toPosition(position, swap, undefined) }
        : null;
    }
    case "LineString": {
      const coords = (leaf.coords ?? []) as Position[];
      return { type: "LineString", coordinates: toPositions(coords, swap, z) };
    }
    case "Polygon": {
      const exterior = (leaf.exterior ?? []) as Position[];
      const interiors = (leaf.interiors ?? []) as Position[][];
      return {
        type: "Polygon",
        coordinates: [
          toRing(exterior, swap, z),
          ...interiors.map((ring) => toRing(ring, swap, z)),
        ],
        ...shadingFields(leaf.appearance, 1),
      };
    }
    case "PolygonMesh": {
      const faces = (leaf.faces ?? []) as unknown[];
      return {
        type: "MultiPolygon",
        coordinates: faces.map((face) => ringsOfFace(face, swap, z)),
        ...shadingFields(leaf.appearance, faces.length),
      };
    }
    case "TriangularMesh": {
      const triangles = (leaf.triangles ?? []) as Position[][];
      return {
        type: "MultiPolygon",
        coordinates: triangles.map((triangle) => [toRing(triangle, swap, z)]),
        ...shadingFields(leaf.appearance, triangles.length),
      };
    }
    case "Solid": {
      // Appearance lives on each shell's mesh, and the shells are frameless —
      // they borrow the solid's frame. Each shell is converted on its own so
      // its own appearance lines up with its own surfaces, then merged.
      const shells = [
        leaf.exterior,
        ...((leaf.interiors ?? []) as unknown[]),
      ].filter(Boolean);
      const converted = shells.map((shell) => {
        const coordinates = polygonsOfShell(shell, swap);
        return {
          type: "MultiPolygon",
          coordinates,
          ...shadingFields(appearanceOfShell(shell), coordinates.length),
        };
      });

      return {
        type: "MultiPolygon",
        coordinates: converted.flatMap(
          (shell) => shell.coordinates as Position[][][],
        ),
        ...mergeShading(converted),
      };
    }
    case "Collection": {
      // Members are bare leaves rather than whole geometries here, so they are
      // wrapped back into the dimension they came from to be resolved.
      const members = (leaf.members ?? []) as unknown[];
      const described = members.map((member) =>
        describeGeometry({ [dimension]: member }),
      );
      const geometries = described
        .map((member) => toGeoJson(member.variant, member.value, dimension))
        .filter(Boolean) as Record<string, unknown>[];

      // A collection has no frame of its own; take it from the members.
      return {
        ...mergeMembers(geometries),
        ...frameOfCollection(described, geometries),
      };
    }
    default:
      return null;
  }
}

/** The multi-part GeoJSON type a homogeneous collection of each type collapses to. */
const MULTI_OF: Record<string, string> = {
  Point: "MultiPoint",
  LineString: "MultiLineString",
  Polygon: "MultiPolygon",
};

/**
 * Collapse a collection into a single multi-part geometry when its members all
 * share a type.
 *
 * That is the common case — a GeoJSON `MultiLineString` arrives as a collection
 * of polylines — and it is worth special-casing, because a `GeometryCollection`
 * carries its members under `geometries` rather than `coordinates`. Everything
 * downstream reads `coordinates`: the table shows it as a column, and it is the
 * shape the legacy transform produced. Only a genuinely mixed collection needs
 * the general form.
 */
function mergeMembers(
  geometries: Record<string, unknown>[],
): Record<string, unknown> {
  if (geometries.length === 0) {
    return { type: "GeometryCollection", geometries: [] };
  }

  const types = new Set(geometries.map((geometry) => geometry.type as string));
  if (types.size === 1) {
    const [type] = [...types];

    const multi = MULTI_OF[type];
    if (multi) {
      return {
        type: multi,
        coordinates: geometries.map((geometry) => geometry.coordinates),
        // A `Polygon` member is one surface, so its binding needs no rebasing
        // beyond the palette offset — the same merge serves both branches.
        ...mergeShading(geometries),
      };
    }

    // Already multi-part — a mesh member became a MultiPolygon — so the parts
    // concatenate rather than nest.
    if (type.startsWith("Multi")) {
      return {
        type,
        coordinates: geometries.flatMap(
          (geometry) => geometry.coordinates as unknown[],
        ),
        ...mergeShading(geometries),
      };
    }
  }

  return { type: "GeometryCollection", geometries };
}

/**
 * Per-member attribute key the CityGML pipeline records a member's level of
 * detail under; see `MEMBER_LOD_KEY` in `citygml_parser/pipeline.rs`.
 */
const MEMBER_LOD_KEY = "lod";

/** A collection converted to GeoJSON, with the level of detail it was cut to. */
type SelectedCollection = {
  geometry: Record<string, unknown> | null;
  /** Present when the members declared a level and one was chosen. */
  lod?: number;
  /** The finest level, when that is not the one drawn. See {@link LodDetail}. */
  detail?: LodDetail;
};

/**
 * Which level of detail to draw, in order of preference.
 *
 * LOD1 first, exactly as the legacy CityGML renderer chose it
 * (`convertFeatureCollectionToPrimitives`: `lod === 1`, then 2, then 3). The
 * order is not "coarsest first" — LOD0 is a *footprint*, a single flat surface
 * at ground level, so preferring it draws a city as flat polygons rather than
 * as buildings. LOD1 is the coarsest level that is still a solid.
 *
 * Anything not listed falls back to the lowest declared level, which keeps a
 * file of nothing but LOD0 drawable.
 */
const LOD_PREFERENCE = [1, 2, 3];

function preferredLod(levels: (number | undefined)[]): number | undefined {
  const declared = levels.filter(
    (level): level is number => level !== undefined,
  );
  if (declared.length === 0) return undefined;

  const preferred = LOD_PREFERENCE.find((level) => declared.includes(level));
  return preferred ?? Math.min(...declared);
}

/**
 * Convert a whole `Geometry` rather than a single leaf.
 *
 * A `GeometryCollection`'s members are themselves whole geometries — including
 * further collections — so conversion has to start at the top of the enum, not
 * at a leaf.
 */
function geoJsonOfDescribed(
  described: GeometryDescription,
): Record<string, unknown> | null {
  if (described.kind === "2d" || described.kind === "3d") {
    const dimension: Dimension =
      described.kind === "2d" ? "Euclidean2D" : "Euclidean3D";
    return toGeoJson(described.variant, described.value, dimension);
  }
  if (described.kind === "collection") {
    return collectionToGeoJson(described.value).geometry;
  }
  return null;
}

/**
 * Frame for a collection, gathered from whichever members carry one.
 *
 * No collection carries a frame — `Collection2D`, `Collection3D` and the
 * top-level `GeometryCollection` all hold only `members` and `attrs`, because
 * members may sit in different frames. Reporting the members' is better than
 * reporting none: it is right whenever they agree, which is the usual case, and
 * the raw view shows the truth when they do not.
 *
 * Falls back to the converted members, which covers a member that is itself a
 * collection and has already resolved its own frame. CityGML reaches two levels
 * this way: a `GeometryCollection` of per-LOD members, each of which may be a
 * `MultiSurface` and so a collection in turn.
 */
function frameOfCollection(
  described: GeometryDescription[],
  converted: Record<string, unknown>[],
): Record<string, unknown> {
  const frames = new Set<string>();

  for (const member of described) {
    const leaf = (member.value ?? {}) as Record<string, unknown>;
    const frame = describeFrame(leaf.frame);
    if (frame) frames.add(frame);
  }

  // A member that is itself a collection has already resolved its own.
  if (frames.size === 0) {
    for (const member of converted) {
      if (typeof member.frame === "string") frames.add(member.frame);
    }
  }

  if (frames.size === 0) return {};
  // Every frame present, not just the first: naming one of several would say
  // the members agree when they do not.
  return { frame: [...frames].join(", ") };
}

/**
 * Convert a top-level geometry collection, choosing one level of detail.
 *
 * This is the shape every CityGML feature takes: one member per `lodN`
 * property, with the level in the collection's parallel `attrs`
 * (`citygml_parser/pipeline.rs`, "so downstream sinks can select a single
 * LOD"). Drawing every member would stack a coarse box model inside a detailed
 * one, so one level is chosen — by {@link LOD_PREFERENCE}.
 */
function collectionToGeoJson(value: unknown): SelectedCollection {
  const collection = (value ?? {}) as Record<string, unknown>;
  const members = (collection.members ?? []) as unknown[];
  const attrs = (collection.attrs ?? []) as Record<string, unknown>[];

  const levels = members.map((_, index) => {
    const lod = attrs[index]?.[MEMBER_LOD_KEY];
    return typeof lod === "number" ? lod : undefined;
  });

  const lod = preferredLod(levels);
  const geometry = convertLevel(members, levels, lod);
  if (!geometry) return { geometry: null, lod };

  return { geometry, lod, detail: finestLevel(members, levels, lod) };
}

/** Convert the members at one level, merged as a single geometry. */
function convertLevel(
  members: unknown[],
  levels: (number | undefined)[],
  lod: number | undefined,
): Record<string, unknown> | null {
  const selected =
    lod === undefined
      ? members
      : members.filter((_, index) => levels[index] === lod);

  const described = selected.map(describeGeometry);
  const geometries = described
    .map(geoJsonOfDescribed)
    .filter(Boolean) as Record<string, unknown>[];

  if (geometries.length === 0) return null;

  return {
    ...mergeMembers(geometries),
    ...frameOfCollection(described, geometries),
  };
}

/**
 * The finest level declared, when the map is not already drawing it.
 *
 * Legacy upgraded a selected feature to LOD3, else LOD2; taking the maximum
 * reaches the same member without hard-coding which levels exist. Returns
 * nothing when the drawn level is already the finest, so a single-level file
 * costs nothing.
 */
function finestLevel(
  members: unknown[],
  levels: (number | undefined)[],
  drawn: number | undefined,
): LodDetail | undefined {
  const declared = levels.filter(
    (level): level is number => level !== undefined,
  );
  if (declared.length === 0) return undefined;

  const finest = Math.max(...declared);
  if (finest === drawn) return undefined;

  const geometry = convertLevel(members, levels, finest);
  return geometry ? { lod: finest, geometry } : undefined;
}

/** A shell's appearance, which sits on the mesh inside it rather than on the shell. */
function appearanceOfShell(shell: unknown): unknown {
  const record = (shell ?? {}) as Record<string, unknown>;
  const mesh = (record.PolygonMesh ?? record.TriangularMesh) as
    | Record<string, unknown>
    | undefined;
  return mesh?.appearance;
}

/** One shell of a solid as GeoJSON polygons. Shells carry no frame of their own. */
function polygonsOfShell(shell: unknown, swap: boolean): Position[][][] {
  const record = (shell ?? {}) as Record<string, unknown>;

  const polygonMesh = record.PolygonMesh as Record<string, unknown> | undefined;
  if (polygonMesh) {
    const faces = (polygonMesh.faces ?? []) as unknown[];
    return faces.map((face) => ringsOfFace(face, swap, undefined));
  }

  const triangularMesh = record.TriangularMesh as
    | Record<string, unknown>
    | undefined;
  if (triangularMesh) {
    const triangles = (triangularMesh.triangles ?? []) as Position[][];
    return triangles.map((triangle) => [toRing(triangle, swap, undefined)]);
  }

  return [];
}

/** Total points across a point cloud's segments, whatever encoding they use. */
function countPointCloud(leaf: Record<string, unknown>): number {
  const segments = (leaf.segments ?? []) as Record<string, unknown>[];
  return segments.reduce((total, segment) => {
    const positions = (segment?.positions ?? {}) as Record<string, unknown>;
    const payload = Object.values(positions)[0];
    if (Array.isArray(payload)) return total + payload.length;
    const scaled = (payload as Record<string, unknown>)?.values;
    return total + (Array.isArray(scaled) ? scaled.length : 0);
  }, 0);
}

/** A count, grouped for the active language — `1,234` and `1.234` differ. */
function num(value: number): string {
  return value.toLocaleString(i18n.language);
}

function summarize(variant: string | null, value: unknown): string | undefined {
  const leaf = (value ?? {}) as Record<string, unknown>;
  const count = (key: string) =>
    Array.isArray(leaf[key]) ? (leaf[key] as unknown[]).length : 0;

  switch (variant) {
    case "Point":
      return i18n.t("Positions: {{n}}", { n: num(1) });
    case "PointCloud":
      return i18n.t("Points: {{n}}", { n: num(countPointCloud(leaf)) });
    case "LineString":
      return i18n.t("Vertices: {{n}}", { n: num(count("coords")) });
    case "Polygon":
      return i18n.t("Vertices: {{n}}, Holes: {{holes}}", {
        n: num(count("exterior")),
        holes: num(count("interiors")),
      });
    case "PolygonMesh":
      return i18n.t("Faces: {{n}}", { n: num(count("faces")) });
    case "TriangularMesh":
      return i18n.t("Triangles: {{n}}", { n: num(count("triangles")) });
    case "Solid":
      return i18n.t("Shells: {{n}}, Voids: {{voids}}", {
        n: num(1),
        voids: num(count("interiors")),
      });
    case "Collection":
      return i18n.t("Members: {{n}}", { n: num(count("members")) });
    case "Csg":
      // The engine's own discriminant — `Union`, `Difference` — not prose.
      return Object.keys(leaf)[0] ?? i18n.t("Boolean combination");
    default:
      return undefined;
  }
}

/**
 * The leaf's coordinate frame, rendered for display.
 *
 * This is the one descriptive field carried alongside real GeoJSON. It is a
 * foreign member, which turf and Cesium both ignore, and it earns that because
 * the CRS moved from one field per feature onto each leaf — without it there is
 * no way to tell at a glance whether coordinates are degrees or metres. It
 * renders a field the engine actually wrote rather than inventing content.
 */
function frameField(described: GeometryDescription): Record<string, unknown> {
  const leaf = (described.value ?? {}) as Record<string, unknown>;
  const frame = describeFrame(leaf.frame);
  return frame ? { frame } : {};
}

/**
 * Stand-in for geometry with no GeoJSON form (a point cloud, a CSG tree).
 *
 * Only here does a summary earn its place: there are no coordinates to read, so
 * without it a row says nothing about whether a cloud holds ten points or ten
 * million. Geometry that does convert carries none — its coordinates already
 * say what a summary would, and inventing prose the engine never wrote makes
 * the derived view disagree with the raw one for no gain.
 */
function toSummaryGeometry(
  described: GeometryDescription,
): Record<string, unknown> {
  const summary = summarize(described.variant, described.value);
  return {
    type: described.label || described.variant || "Unknown",
    ...frameField(described),
    ...(summary ? { summary } : {}),
  };
}

/** Discriminant `Raster` uses for an image that travelled inside the feature. */
const INLINE_RASTER = "InMemory";

/**
 * Whether a value is bulk coordinate data — a nested array bottoming out in
 * numbers.
 *
 * Decided by descending first elements only, so pruning a mesh's `triangles`
 * (a `number[][][]` of every vertex it has) costs three property reads rather
 * than a walk over the whole thing.
 */
function isCoordinateData(value: unknown): boolean {
  let node = value;
  for (let depth = 0; depth < 4; depth++) {
    if (typeof node === "number") return depth > 0;
    if (!Array.isArray(node) || node.length === 0) return false;
    node = node[0];
  }
  return false;
}

/**
 * Replace every inline image in a parsed feature with a note of what it was.
 *
 * A glTF read embeds encoded images in the feature it emits
 * (`Raster::InMemory`), and serde writes those bytes as a JSON array of
 * integers — so after `JSON.parse` a 2 MB texture is a two-million-element JS
 * number array. The reader splits a GLB by `EXT_mesh_features` into one feature
 * per building and re-serializes the texture on each, and the debug panel holds
 * up to 2000 features, so left in place a single textured GLB exhausts the tab.
 *
 * Nothing renders these today, so they are described rather than decoded.
 * Engine PR #2303 left the encoding deliberately open ("UI needs to eagerly
 * decode this to an image, or we need to come up with an alternative"); when
 * that is settled this becomes the place that reads it.
 *
 * Mutates its argument: the feature has just come out of `JSON.parse` and is
 * owned by the caller, and copying it would mean copying the very arrays this
 * exists to drop.
 */
function stripInlineRasters(node: unknown): void {
  if (!node || typeof node !== "object") return;

  if (Array.isArray(node)) {
    if (isCoordinateData(node)) return;
    for (const item of node) stripInlineRasters(item);
    return;
  }

  const record = node as Record<string, unknown>;

  // At a `Raster`, whose sole key is its variant.
  const inline = record[INLINE_RASTER] as Record<string, unknown> | undefined;
  if (inline && Array.isArray(inline.bytes)) {
    record[INLINE_RASTER] = {
      mime_type: inline.mime_type,
      byteLength: inline.bytes.length,
    };
    return;
  }

  for (const value of Object.values(record)) stripInlineRasters(value);
}

/**
 * Transform one parsed new-format JSONL line.
 *
 * The geometry is mutated in place to drop embedded image bytes before anything
 * else looks at it; see {@link stripInlineRasters}.
 */
export function transformNextFeature(parsed: any): TransformedFeature {
  const transformed: TransformedFeature = {
    id: parsed.id,
    type: "Feature",
    properties: { ...parsed.attributes },
  };

  stripInlineRasters(parsed.geometry);

  const described = describeGeometry(parsed.geometry);

  if (described.kind === "none" || described.kind === "unknown") {
    return transformed;
  }

  if (described.kind === "2d" || described.kind === "3d") {
    const dimension: Dimension =
      described.kind === "2d" ? "Euclidean2D" : "Euclidean3D";
    const geoJson = toGeoJson(described.variant, described.value, dimension);
    transformed.geometry = geoJson
      ? { ...geoJson, ...frameField(described) }
      : toSummaryGeometry(described);
    return transformed;
  }

  if (described.kind === "collection") {
    const { geometry, lod, detail } = collectionToGeoJson(described.value);
    // The chosen level is real data the engine wrote, and without it a row
    // gives no clue which of several models the map is drawing.
    transformed.geometry = geometry
      ? { ...geometry, ...(lod !== undefined ? { lod } : {}) }
      : toSummaryGeometry(described);
    if (geometry && detail) transformed.lodDetail = detail;
    return transformed;
  }

  transformed.geometry = toSummaryGeometry(described);
  return transformed;
}
