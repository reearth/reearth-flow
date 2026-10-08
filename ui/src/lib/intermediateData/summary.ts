import { describeGeometry } from "./labels";

/**
 * What a rendered view can make of one feature's geometry, read from the
 * engine's own form before it is converted for display.
 *
 * Kept to a few fields on purpose: the table holds one per row, and the
 * geometry it summarises is not kept.
 */
export type GeometrySummary = {
  /** The top-level type as the engine names it, e.g. "Point (2D)". Empty when there is none. */
  label: string;
  has2D: boolean;
  has3D: boolean;
  /** The EPSG codes the geometry's parts are in, ascending. */
  crs: number[];
  /**
   * Whether some part names no CRS. Every rendered view leaves such a part out:
   * there is nowhere on the globe to put it.
   */
  hasPartWithoutCrs: boolean;
  /** Whether some 3D part names a CRS, which is what a 3D model of the row draws. */
  has3DWithCrs: boolean;
  /** Whether some part is a surface: a polygon, a mesh or a solid. */
  hasSurface: boolean;
  /**
   * Whether some part is a point, line, point cloud or CSG tree. 3D Tiles draw
   * only surfaces, so a map in 3D Tiles leaves these parts out.
   */
  hasNonSurface: boolean;
};

const SURFACES = new Set(["Polygon", "PolygonMesh", "TriangularMesh", "Solid"]);
const NON_SURFACES = new Set(["Point", "PointCloud", "LineString", "Csg"]);

/**
 * The EPSG code a frame places its geometry in, or null when it places it
 * nowhere. A tangent plane is placed by the frame it is anchored in.
 */
const frameEpsg = (frame: unknown): number | null => {
  if (!frame || typeof frame !== "object") return null;
  const record = frame as Record<string, unknown>;
  if (typeof record.Crs === "number") return record.Crs;
  if (record.Tangent && typeof record.Tangent === "object") {
    return frameEpsg((record.Tangent as Record<string, unknown>).base);
  }
  return null;
};

/** Whether `value` is a position, or a list of them, which holds no frames. */
const isCoordinateData = (value: unknown[]): boolean =>
  typeof value[0] === "number" ||
  (Array.isArray(value[0]) && typeof value[0][0] === "number");

export const summarizeGeometry = (geometry: unknown): GeometrySummary => {
  const crs = new Set<number>();
  const summary: GeometrySummary = {
    label: "",
    has2D: false,
    has3D: false,
    crs: [],
    hasPartWithoutCrs: false,
    has3DWithCrs: false,
    hasSurface: false,
    hasNonSurface: false,
  };

  // Every coordinate-bearing part carries its own frame, under whichever of the
  // two embeddings it sits in. Lists of coordinates are skipped whole, so the
  // walk costs one step per part and per ring, not per coordinate.
  const walk = (node: unknown, embedding: "2d" | "3d" | null) => {
    if (Array.isArray(node)) {
      if (node.length === 0 || isCoordinateData(node)) return;
      for (const item of node) walk(item, embedding);
      return;
    }
    if (!node || typeof node !== "object") return;

    for (const [key, value] of Object.entries(node)) {
      if (key === "Euclidean2D") {
        summary.has2D = true;
        walk(value, "2d");
      } else if (key === "Euclidean3D") {
        summary.has3D = true;
        walk(value, "3d");
      } else if (key === "frame") {
        const epsg = frameEpsg(value);
        if (epsg === null) {
          summary.hasPartWithoutCrs = true;
        } else {
          crs.add(epsg);
          if (embedding === "3d") summary.has3DWithCrs = true;
        }
      } else {
        // Inside an embedding, a geometry's variant is its key.
        if (embedding && SURFACES.has(key)) summary.hasSurface = true;
        if (embedding && NON_SURFACES.has(key)) summary.hasNonSurface = true;
        walk(value, embedding);
      }
    }
  };

  walk(geometry, null);

  const described = describeGeometry(geometry);
  summary.label = described.kind === "none" ? "" : described.label;
  summary.crs = [...crs].sort((a, b) => a - b);
  return summary;
};
