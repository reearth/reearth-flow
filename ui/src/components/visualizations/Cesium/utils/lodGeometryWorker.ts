/// <reference lib="webworker" />
import earcut from "earcut";

// ── WGS84 Ellipsoid Constants ────────────────────────────────────────────────
// Matches Cesium.Ellipsoid.WGS84 exactly.
const WGS84_A = 6378137.0; // semi-major axis (meters)
const WGS84_B = 6356752.3142451793; // semi-minor axis (meters)
const WGS84_E2 = (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_A * WGS84_A); // first eccentricity squared

const DEG_TO_RAD = Math.PI / 180;

// ── Types (plain-object mirror of main-thread types) ─────────────────────────

export type WorkerInput = {
  requestId: number;
  polygons: PolygonInput[];
  globalMinZ: number;
};

export type PolygonInput = {
  exterior: CoordInput[];
  globalIndex: number;
  /** RGBA as [r, g, b, a], each 0-1 */
  color: [number, number, number, number];
};

type CoordInput = { x: number; y: number; z: number } | number[];

export type WorkerOutput = {
  requestId: number;
  /** Interleaved XYZ for fill triangles */
  fillPositions: Float64Array;
  /** Triangle indices into fillPositions (every 3 = 1 triangle) */
  fillIndices: Uint32Array;
  /** Per-vertex RGBA for fill (4 floats per vertex, aligned with fillPositions) */
  fillColors: Float32Array;
  /** Interleaved XYZ for outline line segments */
  outlinePositions: Float64Array;
  /** Line indices into outlinePositions (every 2 = 1 segment) */
  outlineIndices: Uint32Array;
};

// ── Coordinate conversion ────────────────────────────────────────────────────

/** Convert geodetic (degrees, height in meters) to ECEF Cartesian XYZ.
 *  Matches Cesium.Cartesian3.fromDegrees for WGS84. */
function geodeticToEcef(
  lonDeg: number,
  latDeg: number,
  height: number,
): [number, number, number] {
  const lon = lonDeg * DEG_TO_RAD;
  const lat = latDeg * DEG_TO_RAD;
  const sinLat = Math.sin(lat);
  const cosLat = Math.cos(lat);
  const sinLon = Math.sin(lon);
  const cosLon = Math.cos(lon);

  const N = WGS84_A / Math.sqrt(1 - WGS84_E2 * sinLat * sinLat);
  const x = (N + height) * cosLat * cosLon;
  const y = (N + height) * cosLat * sinLon;
  const z = (N * (1 - WGS84_E2) + height) * sinLat;
  return [x, y, z];
}

/** Read a coordinate from either {x,y,z} object or [x,y,z] array form. */
function readCoord(c: CoordInput): [number, number, number] {
  if (Array.isArray(c)) {
    return [c[0] ?? 0, c[1] ?? 0, c[2] ?? 0];
  }
  return [c.x ?? 0, c.y ?? 0, c.z ?? 0];
}

// ── Core processing ──────────────────────────────────────────────────────────

function processPolygons(input: WorkerInput): WorkerOutput {
  const { requestId, polygons, globalMinZ } = input;

  // Pre-allocate accumulation arrays (will be compacted into typed arrays)
  const fillVerts: number[] = [];
  const fillIdx: number[] = [];
  const fillCols: number[] = [];
  const outVerts: number[] = [];
  const outIdx: number[] = [];

  let fillVertexCount = 0;
  let outVertexCount = 0;

  for (const polygon of polygons) {
    const ext = polygon.exterior;
    if (!ext || ext.length < 3) continue;

    // ── Convert coordinates to ECEF with height normalization ──────────
    const ecefPositions: number[] = []; // flat XYZ triples
    let validCount = 0;

    for (const coord of ext) {
      const [lon, lat, z] = readCoord(coord);
      if (
        !Number.isFinite(lon) ||
        !Number.isFinite(lat) ||
        lat < -90 ||
        lat > 90 ||
        lon < -180 ||
        lon > 180
      )
        continue;
      // Convert directly to ECEF with normalized height (subtract globalMinZ)
      const finalEcef = geodeticToEcef(lon, lat, z - globalMinZ);
      ecefPositions.push(finalEcef[0], finalEcef[1], finalEcef[2]);
      validCount++;
    }

    if (validCount < 3) continue;

    // ── Triangulate using earcut ──────────────────────────────────────
    // earcut expects 2D coordinates. We project onto the polygon's dominant plane.
    const flatCoords2D = projectToPlane(ecefPositions, validCount);
    const triangles = earcut(flatCoords2D, undefined, 2);

    if (triangles.length === 0) continue;

    // ── Accumulate fill geometry ──────────────────────────────────────
    const fillBaseVertex = fillVertexCount;
    for (let i = 0; i < validCount; i++) {
      fillVerts.push(
        ecefPositions[i * 3],
        ecefPositions[i * 3 + 1],
        ecefPositions[i * 3 + 2],
      );
      fillCols.push(
        polygon.color[0],
        polygon.color[1],
        polygon.color[2],
        polygon.color[3],
      );
      fillVertexCount++;
    }

    for (const triIdx of triangles) {
      fillIdx.push(fillBaseVertex + triIdx);
    }

    // ── Accumulate outline geometry (boundary edges) ──────────────────
    const outBaseVertex = outVertexCount;
    for (let i = 0; i < validCount; i++) {
      outVerts.push(
        ecefPositions[i * 3],
        ecefPositions[i * 3 + 1],
        ecefPositions[i * 3 + 2],
      );
      outVertexCount++;
    }

    for (let i = 0; i < validCount; i++) {
      outIdx.push(outBaseVertex + i, outBaseVertex + ((i + 1) % validCount));
    }
  }

  // ── Pack into typed arrays ──────────────────────────────────────────────
  const fillPositions = new Float64Array(fillVerts);
  const fillIndices = new Uint32Array(fillIdx);
  const fillColors = new Float32Array(fillCols);
  const outlinePositions = new Float64Array(outVerts);
  const outlineIndices = new Uint32Array(outIdx);

  return {
    requestId,
    fillPositions,
    fillIndices,
    fillColors,
    outlinePositions,
    outlineIndices,
  };
}

/**
 * Project 3D ECEF positions onto the polygon's best-fit 2D plane for earcut.
 * Uses the Newell method to find the polygon normal, then projects onto the
 * two axes most perpendicular to the normal.
 */
function projectToPlane(positions: number[], vertexCount: number): number[] {
  // Compute polygon normal via Newell's method
  let nx = 0,
    ny = 0,
    nz = 0;
  for (let i = 0; i < vertexCount; i++) {
    const j = (i + 1) % vertexCount;
    const ix = i * 3,
      jx = j * 3;
    const x0 = positions[ix],
      y0 = positions[ix + 1],
      z0 = positions[ix + 2];
    const x1 = positions[jx],
      y1 = positions[jx + 1],
      z1 = positions[jx + 2];
    nx += (y0 - y1) * (z0 + z1);
    ny += (z0 - z1) * (x0 + x1);
    nz += (x0 - x1) * (y0 + y1);
  }

  // Choose the two axes with the largest extent (drop the dominant normal axis)
  const anx = Math.abs(nx),
    any = Math.abs(ny),
    anz = Math.abs(nz);
  let ax1: number, ax2: number;
  if (anx >= any && anx >= anz) {
    ax1 = 1;
    ax2 = 2; // drop X, project onto YZ
  } else if (any >= anx && any >= anz) {
    ax1 = 0;
    ax2 = 2; // drop Y, project onto XZ
  } else {
    ax1 = 0;
    ax2 = 1; // drop Z, project onto XY
  }

  const result = new Array(vertexCount * 2);
  for (let i = 0; i < vertexCount; i++) {
    result[i * 2] = positions[i * 3 + ax1];
    result[i * 2 + 1] = positions[i * 3 + ax2];
  }
  return result;
}

// ── Worker message handler ───────────────────────────────────────────────────

self.onmessage = (e: MessageEvent<WorkerInput>) => {
  try {
    const result = processPolygons(e.data);

    // Transfer ownership of ArrayBuffers (zero-copy)
    self.postMessage(result, [
      result.fillPositions.buffer,
      result.fillIndices.buffer,
      result.fillColors.buffer,
      result.outlinePositions.buffer,
      result.outlineIndices.buffer,
    ] as unknown as Transferable[]);
  } catch (err) {
    const requestId =
      e?.data && typeof e.data.requestId === "number" ? e.data.requestId : -1;
    self.postMessage({
      requestId,
      error: err instanceof Error ? err.message : String(err),
    });
  }
};
