/**
 * Transform for the engine's new geometry model.
 *
 * The shape differs from the legacy one in three ways that matter here: the
 * geometry is an externally-tagged enum, so its type is a JSON key rather than
 * something to infer from the payload; the CRS moved from one `epsg` per
 * feature onto a `frame` per coordinate-bearing leaf; and coordinates are
 * `[x, y]` arrays instead of `{ x, y }` objects.
 *
 * 2D geometry becomes GeoJSON, because the engine's own view renderer rejects
 * it (`Error::TwoDimensional`) and the 2D map is the only thing that draws it.
 * 3D geometry becomes a summary rather than coordinates: the engine renders 3D
 * itself, into a glb or a 3D Tiles tileset, so reproducing that here would be
 * work thrown away.
 */
import {
  describeGeometry,
  extractAppearance,
  type AppearanceSummary,
  type GeometryDescription,
} from "@flow/lib/intermediateData";

type Position = number[];

export type NextTransformOptions = {
  /** Data URL of the file being read; owns any images lifted out of it. */
  owner: string;
  /** 0-based line number in the source JSONL. */
  rowIndex?: number;
};

export type TransformedFeature = {
  id: string;
  type: "Feature";
  properties: Record<string, unknown>;
  geometry?: unknown;
  /**
   * Line this feature sits at in its file. The engine's view renderer selects
   * by row (`Selection::Row`) and stamps only `rowIndex` on what it renders,
   * so this is the join between a table row and a rendered view.
   */
  rowIndex?: number;
  /**
   * Materials and their texture maps. Held beside the geometry rather than
   * inside it so the table does not grow a column per material, and so the
   * details panel can show the images themselves.
   */
  appearance?: AppearanceSummary;
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

/** GeoJSON requires a closed ring; the engine's validity rules already expect one. */
function closeRing(ring: Position[]): Position[] {
  if (ring.length === 0) return ring;
  const [first] = ring;
  const last = ring[ring.length - 1];
  const closed = first.every((value, axis) => value === last[axis]);
  return closed ? ring : [...ring, first];
}

/** Lift a 2.5D leaf's single elevation onto each of its coordinates. */
function withElevation(coords: Position[], z: unknown): Position[] {
  if (typeof z !== "number") return coords;
  return coords.map((coord) => [...coord, z]);
}

function ringsOfFace(face: unknown, z: unknown): Position[][] {
  const record = (face ?? {}) as Record<string, unknown>;
  const exterior = (record.exterior ?? []) as Position[];
  const holes = (record.holes ?? []) as Position[][];
  return [
    closeRing(withElevation(exterior, z)),
    ...holes.map((hole) => closeRing(withElevation(hole, z))),
  ];
}

/**
 * Convert a 2D leaf to GeoJSON. Meshes flatten to a MultiPolygon — one polygon
 * per face or triangle — which is what the 2D map can draw.
 */
function toGeoJson2D(
  variant: string | null,
  value: unknown,
): Record<string, unknown> | null {
  const leaf = (value ?? {}) as Record<string, unknown>;
  const z = leaf.z;

  switch (variant) {
    case "Point": {
      const position = leaf.position as Position | undefined;
      return position ? { type: "Point", coordinates: position } : null;
    }
    case "LineString": {
      const coords = (leaf.coords ?? []) as Position[];
      return { type: "LineString", coordinates: withElevation(coords, z) };
    }
    case "Polygon": {
      const exterior = (leaf.exterior ?? []) as Position[];
      const interiors = (leaf.interiors ?? []) as Position[][];
      return {
        type: "Polygon",
        coordinates: [
          closeRing(withElevation(exterior, z)),
          ...interiors.map((ring) => closeRing(withElevation(ring, z))),
        ],
      };
    }
    case "PolygonMesh": {
      const faces = (leaf.faces ?? []) as unknown[];
      return {
        type: "MultiPolygon",
        coordinates: faces.map((face) => ringsOfFace(face, z)),
      };
    }
    case "TriangularMesh": {
      const triangles = (leaf.triangles ?? []) as Position[][];
      return {
        type: "MultiPolygon",
        coordinates: triangles.map((triangle) => [
          closeRing(withElevation(triangle, z)),
        ]),
      };
    }
    case "Collection": {
      // A collection's members may differ in type, so the general GeoJSON form
      // is the safe one; turf and Cesium both accept it.
      const members = (leaf.members ?? []) as unknown[];
      const geometries = members
        .map((member) => {
          const described = describeGeometry({ Euclidean2D: member });
          return toGeoJson2D(described.variant, described.value);
        })
        .filter(Boolean);
      return { type: "GeometryCollection", geometries };
    }
    default:
      return null;
  }
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
 * Descriptive stand-in for geometry that is not turned into GeoJSON. Keyed on
 * a small, stable set of fields so the table gets a handful of useful columns
 * rather than one per geometry variant in the file.
 */
function toSummaryGeometry(
  described: GeometryDescription,
  textureCount: number,
): Record<string, unknown> {
  const leaf = (described.value ?? {}) as Record<string, unknown>;
  const summary = summarize(described.variant, described.value);
  const frame = describeFrame(leaf.frame);

  return {
    type: described.label || described.variant || "Unknown",
    ...(frame ? { frame } : {}),
    ...(summary ? { summary } : {}),
    ...(textureCount > 0 ? { textures: textureCount } : {}),
  };
}

/**
 * Transform one parsed new-format JSONL line.
 *
 * The geometry is mutated in place to remove embedded image bytes before
 * anything else looks at it; see `extractAppearance`.
 */
export function transformNextFeature(
  parsed: any,
  options: NextTransformOptions,
): TransformedFeature {
  const transformed: TransformedFeature = {
    id: parsed.id,
    type: "Feature",
    properties: { ...parsed.attributes },
  };

  if (options.rowIndex !== undefined) transformed.rowIndex = options.rowIndex;

  const appearance = extractAppearance(parsed.geometry, options.owner);
  const textures = appearance.textures;
  if (appearance.materials.length > 0) transformed.appearance = appearance;

  const described = describeGeometry(parsed.geometry);

  if (described.kind === "none" || described.kind === "unknown") {
    return transformed;
  }

  if (described.kind === "2d") {
    const geoJson = toGeoJson2D(described.variant, described.value);
    const leaf = (described.value ?? {}) as Record<string, unknown>;
    const frame = describeFrame(leaf.frame);
    transformed.geometry = geoJson
      ? { ...geoJson, ...(frame ? { frame } : {}) }
      : toSummaryGeometry(described, textures.length);
    return transformed;
  }

  transformed.geometry = toSummaryGeometry(described, textures.length);
  return transformed;
}
