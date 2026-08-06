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
import {
  describeGeometry,
  type GeometryDescription,
} from "@flow/lib/intermediateData";

type Position = number[];

export type TransformedFeature = {
  id: string;
  type: "Feature";
  properties: Record<string, unknown>;
  geometry?: unknown;
  /**
   * The parsed JSONL record as the engine wrote it, inline image bytes
   * excepted. The details panel shows this rather than the derived geometry,
   * so debugging sees the engine's own structure.
   */
  source?: unknown;
};

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
 * The engine resolves this through PROJ, which the browser has no equivalent
 * of, so this is a list of the systems Flow actually reads. Anything not named
 * here passes through east-first, which is the common case (Web Mercator, UTM,
 * and every projected CRS outside Japan).
 */
function isNorthFirst(epsg: number): boolean {
  // Geographic CRSs: WGS 84, JGD2000 and JGD2011, with their 3D forms.
  // EPSG:6697 (JGD2011 + height) is what PLATEAU CityGML carries.
  if ([4326, 4979, 4612, 6667, 6668, 6697].includes(epsg)) return true;
  // Japan Plane Rectangular CS zones I-XIX, whose axes are (X = north, Y = east).
  if (epsg >= 2443 && epsg <= 2461) return true; // JGD2000
  if (epsg >= 6669 && epsg <= 6687) return true; // JGD2011
  if (epsg >= 30161 && epsg <= 30179) return true; // Tokyo datum
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
 * When neither applies the original array is returned rather than copied. The
 * feature keeps its source geometry for raw inspection, so a copy here would
 * mean holding every coordinate twice; nothing downstream mutates these.
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
 * Convert a geometry leaf to GeoJSON.
 *
 * 2D and 3D leaves use the same field names and differ only in coordinate
 * arity, and a GeoJSON position takes an optional third element, so one
 * conversion serves both. Meshes and solids flatten to a MultiPolygon — one
 * polygon per face or triangle — which is what Cesium draws.
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
      };
    }
    case "PolygonMesh": {
      const faces = (leaf.faces ?? []) as unknown[];
      return {
        type: "MultiPolygon",
        coordinates: faces.map((face) => ringsOfFace(face, swap, z)),
      };
    }
    case "TriangularMesh": {
      const triangles = (leaf.triangles ?? []) as Position[][];
      return {
        type: "MultiPolygon",
        coordinates: triangles.map((triangle) => [toRing(triangle, swap, z)]),
      };
    }
    case "Solid": {
      // Appearance lives on each shell's mesh, and the shells are frameless —
      // they borrow the solid's frame.
      const shells = [
        leaf.exterior,
        ...((leaf.interiors ?? []) as unknown[]),
      ].filter(Boolean);
      return {
        type: "MultiPolygon",
        coordinates: shells.flatMap((shell) => polygonsOfShell(shell, swap)),
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
};

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
 * Frame for a collection, taken from the first member that has one.
 *
 * No collection carries a frame — `Collection2D`, `Collection3D` and the
 * top-level `GeometryCollection` all hold only `members` and `attrs`, because
 * members may sit in different frames. Reporting the first is better than
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
 * one, so the lowest level present wins — the same preference the legacy
 * CityGML renderer applied.
 */
function collectionToGeoJson(value: unknown): SelectedCollection {
  const collection = (value ?? {}) as Record<string, unknown>;
  const members = (collection.members ?? []) as unknown[];
  const attrs = (collection.attrs ?? []) as Record<string, unknown>[];

  const levels = members.map((_, index) => {
    const lod = attrs[index]?.[MEMBER_LOD_KEY];
    return typeof lod === "number" ? lod : undefined;
  });

  const declared = levels.filter(
    (level): level is number => level !== undefined,
  );
  const lod = declared.length ? Math.min(...declared) : undefined;

  const selected =
    lod === undefined
      ? members
      : members.filter((_, index) => levels[index] === lod);

  const described = selected.map(describeGeometry);
  const geometries = described
    .map(geoJsonOfDescribed)
    .filter(Boolean) as Record<string, unknown>[];

  if (geometries.length === 0) return { geometry: null, lod };

  return {
    geometry: {
      ...mergeMembers(geometries),
      ...frameOfCollection(described, geometries),
    },
    lod,
  };
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

/** Short "what is in here" line for geometry the UI does not draw itself. */
function summarize(variant: string | null, value: unknown): string | undefined {
  const leaf = (value ?? {}) as Record<string, unknown>;
  const count = (key: string) =>
    Array.isArray(leaf[key]) ? (leaf[key] as unknown[]).length : 0;

  switch (variant) {
    case "Point":
      return "1 position";
    case "PointCloud":
      return `${countPointCloud(leaf).toLocaleString()} points`;
    case "LineString":
      return `${count("coords")} vertices`;
    case "Polygon":
      return `${count("exterior")} vertices, ${count("interiors")} holes`;
    case "PolygonMesh":
      return `${count("faces")} faces`;
    case "TriangularMesh":
      return `${count("triangles")} triangles`;
    case "Solid":
      return `1 exterior shell, ${count("interiors")} voids`;
    case "Collection":
      return `${count("members")} members`;
    case "Csg":
      return Object.keys(leaf)[0] ?? "boolean combination";
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
    const { geometry, lod } = collectionToGeoJson(described.value);
    // The chosen level is real data the engine wrote, and without it a row
    // gives no clue which of several models the map is drawing.
    transformed.geometry = geometry
      ? { ...geometry, ...(lod !== undefined ? { lod } : {}) }
      : toSummaryGeometry(described);
    return transformed;
  }

  transformed.geometry = toSummaryGeometry(described);
  return transformed;
}
