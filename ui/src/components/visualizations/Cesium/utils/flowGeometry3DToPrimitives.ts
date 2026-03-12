import {
  BoundingSphere,
  Cartesian3,
  Color,
  ColorGeometryInstanceAttribute,
  ComponentDatatype,
  Geometry,
  GeometryAttribute,
  GeometryAttributes,
  GeometryInstance,
  PerInstanceColorAppearance,
  PolylineColorAppearance,
  PolylineGeometry,
  Primitive,
  PrimitiveType,
  ShowGeometryInstanceAttribute,
} from "cesium";

// ── Types ────────────────────────────────────────────────────────────────────

export type TriangularMeshData = {
  vertices: { x: number; y: number; z: number }[];
  triangles: number[][];
};

export type LinePoint = { x: number; y: number; z: number };
export type LineSegment = { start: LinePoint; end: LinePoint };

export type FlowGeometry3DFeature = {
  id?: string;
  type: "Feature";
  properties: Record<string, any>;
  geometry: {
    type: "FlowGeometry3D";
    triangularMesh?: TriangularMeshData;
    line?: LineSegment;
    [key: string]: any;
  };
};

export type FlowGeometry3DFeatureInstanceData = {
  feature: FlowGeometry3DFeature;
  meshInstanceIds: object[];
  lineInstanceIds: object[];
};

export type FlowGeometry3DPrimitivesResult = {
  meshPrimitive: Primitive | null;
  linePrimitive: Primitive | null;
  featureMap: Map<string, FlowGeometry3DFeatureInstanceData>;
  boundingSphere: BoundingSphere | null;
};

// Plain object used as GeometryInstance id — returned directly by scene.pick()
type InstanceId = {
  _originalId: string;
  featureId: string;
  instanceId: string;
};

// ── Helpers ──────────────────────────────────────────────────────────────────

function boundingSphereFromPositionsTyped(
  positions: Float64Array,
): BoundingSphere {
  let minX = Infinity,
    minY = Infinity,
    minZ = Infinity;
  let maxX = -Infinity,
    maxY = -Infinity,
    maxZ = -Infinity;

  for (let i = 0; i < positions.length; i += 3) {
    const x = positions[i];
    const y = positions[i + 1];
    const z = positions[i + 2];
    if (x < minX) minX = x;
    if (y < minY) minY = y;
    if (z < minZ) minZ = z;
    if (x > maxX) maxX = x;
    if (y > maxY) maxY = y;
    if (z > maxZ) maxZ = z;
  }

  const cx = (minX + maxX) * 0.5;
  const cy = (minY + maxY) * 0.5;
  const cz = (minZ + maxZ) * 0.5;

  let r2 = 0;
  for (let i = 0; i < positions.length; i += 3) {
    const dx = positions[i] - cx;
    const dy = positions[i + 1] - cy;
    const dz = positions[i + 2] - cz;
    const d2 = dx * dx + dy * dy + dz * dz;
    if (d2 > r2) r2 = d2;
  }

  return new BoundingSphere(new Cartesian3(cx, cy, cz), Math.sqrt(r2));
}

const MAX_MESH_SAMPLE_POSITIONS = 20;

/**
 * Sample up to MAX_MESH_SAMPLE_POSITIONS evenly-distributed vertices from a
 * TriangularMesh and push Cartesian3 positions into the output array.
 */
export function sampleMeshPositions(
  mesh: TriangularMeshData,
  out: Cartesian3[],
  outCap: number,
): void {
  if (mesh.vertices.length === 0 || out.length >= outCap) return;
  const step = Math.max(
    1,
    Math.floor(mesh.vertices.length / MAX_MESH_SAMPLE_POSITIONS),
  );
  for (
    let vi = 0;
    vi < mesh.vertices.length && out.length < outCap;
    vi += step
  ) {
    const { x, y, z } = mesh.vertices[vi];
    out.push(Cartesian3.fromDegrees(x, y, z));
  }
}

/**
 * Build a GeometryInstance from an already-triangulated TriangularMesh.
 * Vertices must be in WGS84 (x = longitude°, y = latitude°, z = height m).
 * Per-vertex normals are computed from averaged face normals in ECEF space.
 */
export function triangularMeshToGeometryInstance(
  mesh: TriangularMeshData,
  color: Color,
  id: object,
): GeometryInstance | null {
  if (mesh.vertices.length < 3 || mesh.triangles.length === 0) return null;

  const positions = new Float64Array(mesh.vertices.length * 3);
  for (let i = 0; i < mesh.vertices.length; i++) {
    const { x, y, z } = mesh.vertices[i];
    const cart = Cartesian3.fromDegrees(x, y, z);
    positions[i * 3] = cart.x;
    positions[i * 3 + 1] = cart.y;
    positions[i * 3 + 2] = cart.z;
  }

  const indices = new Uint32Array(mesh.triangles.length * 3);
  for (let i = 0; i < mesh.triangles.length; i++) {
    indices[i * 3] = mesh.triangles[i][0];
    indices[i * 3 + 1] = mesh.triangles[i][1];
    indices[i * 3 + 2] = mesh.triangles[i][2];
  }

  // Compute per-vertex normals by averaging face normals in ECEF space
  const normalAccum = new Float64Array(mesh.vertices.length * 3);
  for (const triangle of mesh.triangles) {
    const i0 = triangle[0];
    const i1 = triangle[1];
    const i2 = triangle[2];
    const ax = positions[i1 * 3] - positions[i0 * 3];
    const ay = positions[i1 * 3 + 1] - positions[i0 * 3 + 1];
    const az = positions[i1 * 3 + 2] - positions[i0 * 3 + 2];
    const bx = positions[i2 * 3] - positions[i0 * 3];
    const by = positions[i2 * 3 + 1] - positions[i0 * 3 + 1];
    const bz = positions[i2 * 3 + 2] - positions[i0 * 3 + 2];
    const nx = ay * bz - az * by;
    const ny = az * bx - ax * bz;
    const nz = ax * by - ay * bx;
    for (const vi of [i0, i1, i2]) {
      normalAccum[vi * 3] += nx;
      normalAccum[vi * 3 + 1] += ny;
      normalAccum[vi * 3 + 2] += nz;
    }
  }
  const normals = new Float32Array(mesh.vertices.length * 3);
  for (let i = 0; i < mesh.vertices.length; i++) {
    const nx = normalAccum[i * 3];
    const ny = normalAccum[i * 3 + 1];
    const nz = normalAccum[i * 3 + 2];
    const len = Math.sqrt(nx * nx + ny * ny + nz * nz);
    if (len > 0) {
      normals[i * 3] = nx / len;
      normals[i * 3 + 1] = ny / len;
      normals[i * 3 + 2] = nz / len;
    }
  }

  const geomAttributes = new GeometryAttributes();
  geomAttributes.position = new GeometryAttribute({
    componentDatatype: ComponentDatatype.DOUBLE,
    componentsPerAttribute: 3,
    values: positions,
  });
  geomAttributes.normal = new GeometryAttribute({
    componentDatatype: ComponentDatatype.FLOAT,
    componentsPerAttribute: 3,
    values: normals,
  });

  const geometry = new Geometry({
    attributes: geomAttributes,
    indices,
    primitiveType: PrimitiveType.TRIANGLES,
    boundingSphere: boundingSphereFromPositionsTyped(positions),
  });

  return new GeometryInstance({
    id,
    geometry,
    attributes: {
      color: ColorGeometryInstanceAttribute.fromColor(color),
      show: new ShowGeometryInstanceAttribute(true),
    },
  });
}

/**
 * Build a GeometryInstance from a line segment {start, end} in WGS84 (lon°, lat°, height m).
 */
export function lineToGeometryInstance(
  segment: LineSegment,
  color: Color,
  id: object,
): GeometryInstance | null {
  const positions = [
    Cartesian3.fromDegrees(segment.start.x, segment.start.y, segment.start.z),
    Cartesian3.fromDegrees(segment.end.x, segment.end.y, segment.end.z),
  ];

  return new GeometryInstance({
    id,
    geometry: new PolylineGeometry({
      positions,
      width: 2.0,
      vertexFormat: PolylineColorAppearance.VERTEX_FORMAT,
    }),
    attributes: {
      color: ColorGeometryInstanceAttribute.fromColor(color),
      show: new ShowGeometryInstanceAttribute(true),
    },
  });
}

// ── Main function ─────────────────────────────────────────────────────────────

/**
 * Convert a collection of FlowGeometry3D features into batched Cesium Primitives.
 * Handles triangularMesh and line geometry types.
 */
export function convertFlowGeometry3DCollectionToPrimitives(
  features: FlowGeometry3DFeature[],
): FlowGeometry3DPrimitivesResult {
  const meshInstances: GeometryInstance[] = [];
  const lineInstances: GeometryInstance[] = [];
  const featureMap = new Map<string, FlowGeometry3DFeatureInstanceData>();
  const sampledPositions: Cartesian3[] = [];

  for (const feature of features) {
    const { geometry, properties } = feature;
    const featureId: string = properties?._originalId || feature.id || "";
    if (!featureId) continue;

    const entry: FlowGeometry3DFeatureInstanceData = {
      feature,
      meshInstanceIds: [],
      lineInstanceIds: [],
    };

    if (geometry.triangularMesh) {
      const mesh = geometry.triangularMesh as TriangularMeshData;
      const color = Color.CYAN.withAlpha(0.8);
      const instanceId: InstanceId = {
        _originalId: featureId,
        featureId,
        instanceId: `${featureId}_mesh_0`,
      };
      const instance = triangularMeshToGeometryInstance(
        mesh,
        color,
        instanceId,
      );
      if (instance) {
        meshInstances.push(instance);
        entry.meshInstanceIds.push(instanceId);
        sampleMeshPositions(mesh, sampledPositions, 500);
      }
    } else if (geometry.line) {
      const segment = geometry.line as LineSegment;
      const color = Color.YELLOW.withAlpha(0.9);
      const instanceId: InstanceId = {
        _originalId: featureId,
        featureId,
        instanceId: `${featureId}_line_0`,
      };
      const instance = lineToGeometryInstance(segment, color, instanceId);
      if (instance) {
        lineInstances.push(instance);
        entry.lineInstanceIds.push(instanceId);
        if (sampledPositions.length < 500) {
          sampledPositions.push(
            Cartesian3.fromDegrees(
              segment.start.x,
              segment.start.y,
              segment.start.z,
            ),
          );
        }
      }
    }

    featureMap.set(featureId, entry);
  }

  // asynchronous: false — custom Geometry has no _workerName, must be synchronous
  const meshPrimitive =
    meshInstances.length > 0
      ? new Primitive({
          geometryInstances: meshInstances,
          appearance: new PerInstanceColorAppearance({
            translucent: true,
            flat: false,
            closed: false,
          }),
          asynchronous: false,
        })
      : null;

  const linePrimitive =
    lineInstances.length > 0
      ? new Primitive({
          geometryInstances: lineInstances,
          appearance: new PolylineColorAppearance(),
          asynchronous: false,
        })
      : null;

  const boundingSphere =
    sampledPositions.length > 0
      ? BoundingSphere.fromPoints(sampledPositions)
      : null;

  return { meshPrimitive, linePrimitive, featureMap, boundingSphere };
}
