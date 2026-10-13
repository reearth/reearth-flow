export type HeightRange = { min: number; max: number };

/**
 * CRSs whose first two (displayed) coordinates are longitude and latitude in
 * degrees, so a position in them can be placed on the globe as it is.
 */
const GEOGRAPHIC_CRS = new Set([4326, 4979, 4612, 6668, 6697]);

export const isGeographicCrs = (epsg: number) => GEOGRAPHIC_CRS.has(epsg);

/**
 * The lowest and highest heights a displayed (GeoJSON) geometry reaches, from
 * the third element of its positions. Null when it has no heights, as a 2D
 * geometry does.
 *
 * Walks every position, so it is meant for one feature at a time, such as
 * the selected one, not for every row.
 */
export const heightRange = (geometry: unknown): HeightRange | null => {
  let min = Infinity;
  let max = -Infinity;

  const visitPositions = (node: unknown) => {
    if (!Array.isArray(node)) return;
    if (typeof node[0] === "number") {
      const height = node[2];
      if (typeof height === "number" && Number.isFinite(height)) {
        if (height < min) min = height;
        if (height > max) max = height;
      }
      return;
    }
    for (const item of node) visitPositions(item);
  };

  const visitGeometry = (node: unknown) => {
    if (!node || typeof node !== "object") return;
    const record = node as Record<string, unknown>;
    visitPositions(record.coordinates);
    if (Array.isArray(record.geometries)) {
      for (const member of record.geometries) visitGeometry(member);
    }
  };

  visitGeometry(geometry);
  return min <= max ? { min, max } : null;
};

/** Where a geometry lies, as the extent of its first two coordinates. */
export type Bounds = {
  west: number;
  south: number;
  east: number;
  north: number;
  /** Its height range, when it has heights. */
  heights: HeightRange | null;
};

/**
 * The extent of a displayed (GeoJSON) geometry. In a geographic CRS (see
 * {@link isGeographicCrs}) that is longitude and latitude, which places it on
 * the globe. Null when it has no positions. Walks every position, so it is
 * meant for one feature at a time.
 */
export const geometryBounds = (geometry: unknown): Bounds | null => {
  let west = Infinity;
  let south = Infinity;
  let east = -Infinity;
  let north = -Infinity;

  const visitPositions = (node: unknown) => {
    if (!Array.isArray(node)) return;
    if (typeof node[0] === "number") {
      const [x, y] = node;
      if (Number.isFinite(x) && Number.isFinite(y)) {
        west = Math.min(west, x);
        east = Math.max(east, x);
        south = Math.min(south, y);
        north = Math.max(north, y);
      }
      return;
    }
    for (const item of node) visitPositions(item);
  };

  const visitGeometry = (node: unknown) => {
    if (!node || typeof node !== "object") return;
    const record = node as Record<string, unknown>;
    visitPositions(record.coordinates);
    if (Array.isArray(record.geometries)) {
      for (const member of record.geometries) visitGeometry(member);
    }
  };

  visitGeometry(geometry);
  if (west > east) return null;
  return { west, south, east, north, heights: heightRange(geometry) };
};
