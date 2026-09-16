/**
 * Cesium reads every coordinate we hand it as WGS84 (EPSG:4326) degrees.
 *
 * Projected data — e.g. JGD2011 / Japan Plane Rectangular CS, whose ordinates
 * are metres in the tens of thousands — therefore turns each metre into a
 * degree, and a small footprint becomes a polygon spanning thousands of
 * kilometres once wrapped onto the globe. Cesium then subdivides those edges
 * down to its ~1° granularity (PolygonPipeline.computeRhumbLineSubdivision),
 * which grows the vertex arrays until it throws
 * `RangeError: Invalid array length` — or exhausts the tab's memory first and
 * takes the whole app down with it.
 *
 * The viewer only supports EPSG:4326, so anything that cannot be a lon/lat
 * pair is rejected before it reaches Cesium.
 */

export const isValidWgs84Position = (lon: unknown, lat: unknown): boolean =>
  typeof lon === "number" &&
  typeof lat === "number" &&
  Number.isFinite(lon) &&
  Number.isFinite(lat) &&
  lon >= -180 &&
  lon <= 180 &&
  lat >= -90 &&
  lat <= 90;

/**
 * Recursively validate a GeoJSON `coordinates` member (a position, or any
 * depth of nested position arrays). A single bad position invalidates the
 * whole member: dropping only the offending vertices would leave a ring that
 * is still wrong but no longer obviously so.
 */
export const isValidWgs84Coordinates = (coords: unknown): boolean => {
  if (!Array.isArray(coords) || coords.length === 0) return false;

  if (typeof coords[0] === "number") {
    return isValidWgs84Position(coords[0], coords[1]);
  }

  return coords.every((child) => isValidWgs84Coordinates(child));
};

/** How many positions to inspect before deciding a dataset's CRS. */
const CRS_SAMPLE_LIMIT = 500;

type SampleState = { checked: number; invalid: boolean };

/**
 * Walk an arbitrary geometry payload — GeoJSON `coordinates` arrays as well as
 * the `{ x, y, z }` vertices used by CityGmlGeometry — collecting positions
 * until one is invalid or the sample limit is reached.
 */
const sampleGeometry = (node: any, state: SampleState, limit: number): void => {
  if (state.invalid || state.checked >= limit || node == null) return;

  if (Array.isArray(node)) {
    if (typeof node[0] === "number") {
      state.checked++;
      if (!isValidWgs84Position(node[0], node[1])) state.invalid = true;
      return;
    }
    for (const child of node) sampleGeometry(child, state, limit);
    return;
  }

  if (typeof node === "object") {
    if (typeof node.x === "number" && typeof node.y === "number") {
      state.checked++;
      if (!isValidWgs84Position(node.x, node.y)) state.invalid = true;
      return;
    }
    for (const value of Object.values(node)) {
      if (value != null && typeof value === "object") {
        sampleGeometry(value, state, limit);
      }
    }
  }
};

/**
 * True when a geometry holds sampled coordinates that cannot be WGS84 lon/lat —
 * the signature of data still in a projected CRS.
 *
 * Sampling is bounded, so this decides a dataset's CRS cheaply but does not
 * prove every position is safe. Use {@link hasUnsupportedCrs} to gate what
 * reaches Cesium.
 */
export const hasNonWgs84Geometry = (
  geometry: unknown,
  limit: number = CRS_SAMPLE_LIMIT,
): boolean => {
  const state: SampleState = { checked: 0, invalid: false };
  sampleGeometry(geometry, state, limit);
  return state.invalid;
};

/**
 * True when a feature's geometry must be kept away from Cesium.
 *
 * Plain GeoJSON is checked exhaustively: nothing downstream re-validates it,
 * and one bad vertex anywhere in a long ring is enough to blow up the rhumb
 * subdivision. Other payloads — CityGmlGeometry, and the raw FlowGeometry3D
 * fallback the intermediate-data transform emits — are sampled instead:
 * `coordsToPositions` range-checks every CityGML vertex on its way to a
 * `Cartesian3`, so the sample only has to be good enough to explain the dataset
 * to the user.
 *
 * A feature carrying no geometry is renderable; Cesium simply draws nothing.
 */
export const hasUnsupportedCrs = (geometry: any): boolean => {
  if (geometry == null) return false;

  return geometry.coordinates !== undefined
    ? !isValidWgs84Coordinates(geometry.coordinates)
    : hasNonWgs84Geometry(geometry);
};
