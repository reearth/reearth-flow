import {
  BoundingSphere,
  Cartesian3,
  Cartographic,
  Color,
  ColorGeometryInstanceAttribute,
  ComponentDatatype,
  Geometry,
  GeometryAttribute,
  GeometryAttributes,
  GeometryInstance,
  GroundPrimitive,
  Matrix3,
  Matrix4,
  PerInstanceColorAppearance,
  PolygonGeometry,
  PolygonHierarchy,
  PolygonOutlineGeometry,
  PolylineColorAppearance,
  PolylineGeometry,
  Primitive,
  PrimitiveCollection,
  PrimitiveType,
  ShowGeometryInstanceAttribute,
  Transforms,
} from "cesium";
import proj4 from "proj4";

// ── Types ────────────────────────────────────────────────────────────────────

type CityGmlGeometry = {
  type: "CityGmlGeometry";
  [key: string]: any;
};

type CityGmlFeature = {
  id?: string;
  type: "Feature";
  properties: Record<string, any>;
  geometry: CityGmlGeometry;
};

export type CityGmlTypeConfig = {
  detect: (props: Record<string, any>) => boolean;
  displayName: string;
  color: Color;
  useSurfaceTypeColors?: boolean;
  attrKeys: string[];
};

type TriangularMeshData = {
  vertices: { x: number; y: number; z: number }[];
  triangles: number[][];
};

type LinePoint = { x: number; y: number; z: number };
type LineSegment = { start: LinePoint; end: LinePoint };

// Plain object used as GeometryInstance id — returned directly by scene.pick()
type InstanceId = {
  _originalId: string;
  featureId: string;
  instanceId: string;
};

export type FeatureInstanceData = {
  feature: CityGmlFeature;
  /** GeometryInstance id objects stored in the shared absolutePrimitive */
  absoluteInstanceIds: object[];
  /** GeometryInstance id objects stored in the shared groundPrimitive */
  groundInstanceIds: object[];
  /** GeometryInstance id objects stored in the shared meshPrimitive */
  meshInstanceIds: object[];
  /** GeometryInstance id objects stored in the shared linePrimitive */
  lineInstanceIds: object[];
  lodPrimitiveCollection: PrimitiveCollection | null;
};

export type PrimitivesResult = {
  absolutePrimitive: Primitive | null;
  groundPrimitive: GroundPrimitive | null;
  /** Flat-shaded primitive for TriangularMesh geometry (no vertex normals required) */
  meshPrimitive: Primitive | null;
  /** Polyline primitive for line geometry */
  linePrimitive: Primitive | null;
  featureMap: Map<string, FeatureInstanceData>;
  boundingSphere: BoundingSphere | null;
};

// ── Known 3D CityGML feature types ──────────────────────────────────────────

export const CITYGML_3D_TYPES: CityGmlTypeConfig[] = [
  {
    displayName: "Building",
    color: Color.BLUE.withAlpha(0.8),
    useSurfaceTypeColors: true,
    attrKeys: [
      "bldg:measuredHeight",
      "bldg:usage",
      "bldg:class",
      "bldg:yearOfConstruction",
    ],
    detect: (p) =>
      !!(
        p?.["bldg:measuredHeight"] ||
        p?.["bldg:usage"] ||
        p?.["bldg:class"] ||
        p?.gmlName?.includes("bldg:") ||
        p?.cityGmlAttributes?.["bldg:measuredHeight"] ||
        p?.cityGmlAttributes?.["bldg:usage"] ||
        p?.cityGmlAttributes?.["bldg:class"]
      ),
  },
  {
    displayName: "Transportation",
    color: Color.DIMGRAY.withAlpha(0.85),
    attrKeys: ["tran:class", "tran:function", "tran:usage"],
    detect: (p) =>
      !!(
        p?.gmlName?.includes("tran:") ||
        p?.featureType?.includes("tran:") ||
        p?.metadata?.featureType?.includes("tran:") ||
        p?.["tran:class"] ||
        p?.["tran:function"] ||
        p?.cityGmlAttributes?.["tran:class"] ||
        p?.cityGmlAttributes?.["tran:function"]
      ),
  },
  {
    displayName: "Bridge",
    color: Color.SLATEGRAY.withAlpha(0.85),
    attrKeys: [
      "brid:class",
      "brid:function",
      "brid:usage",
      "brid:yearOfConstruction",
    ],
    detect: (p) =>
      !!(
        p?.gmlName?.includes("brid:") ||
        p?.featureType?.includes("brid:") ||
        p?.metadata?.featureType?.includes("brid:") ||
        p?.["brid:class"] ||
        p?.cityGmlAttributes?.["brid:class"]
      ),
  },
  {
    displayName: "City Furniture",
    color: Color.DARKKHAKI.withAlpha(0.85),
    attrKeys: ["frn:class", "frn:function", "frn:usage"],
    detect: (p) =>
      !!(
        p?.gmlName?.includes("frn:") ||
        p?.featureType?.includes("frn:") ||
        p?.metadata?.featureType?.includes("frn:") ||
        p?.["frn:class"] ||
        p?.cityGmlAttributes?.["frn:class"]
      ),
  },
  {
    displayName: "Vegetation",
    color: Color.FORESTGREEN.withAlpha(0.75),
    attrKeys: ["veg:class", "veg:function", "veg:species", "veg:height"],
    detect: (p) =>
      !!(
        p?.gmlName?.includes("veg:") ||
        p?.featureType?.includes("veg:") ||
        p?.metadata?.featureType?.includes("veg:") ||
        p?.["veg:class"] ||
        p?.cityGmlAttributes?.["veg:class"]
      ),
  },
];

// ── JGD2011 Zone IX → WGS84 coordinate conversion ───────────────────────────
// EPSG:6677 — Japan Plane Rectangular CS Zone IX
// Engine stores x = easting (m), y = northing (m); proj4 expects (easting, northing)

const JGD2011_ZONE9 =
  "+proj=tmerc +lat_0=36 +lon_0=139.8333333333333 +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs";

function jgd2011Zone9ToCartesian3(x: number, y: number, z: number): Cartesian3 {
  const [lon, lat] = proj4(JGD2011_ZONE9, "WGS84", [x, y]);
  return Cartesian3.fromDegrees(lon, lat, z);
}

// ── Internal helpers ─────────────────────────────────────────────────────────

const MAX_DEM_POLYGONS = 150;

function isDemFeature(properties: Record<string, any>): boolean {
  return !!(
    properties?.gmlName?.includes("dem:") ||
    properties?.featureType?.includes("dem:") ||
    properties?.metadata?.featureType?.includes("dem:") ||
    properties?.["dem:class"] ||
    properties?.["dem:type"] ||
    properties?.cityGmlAttributes?.["dem:class"] ||
    properties?.cityGmlAttributes?.["dem:type"]
  );
}

function coordsToPositions(coordinates: any[]): Cartesian3[] {
  return coordinates
    .filter((coord) => coord != null)
    .map((coord) => {
      if (
        typeof coord === "object" &&
        coord.x !== undefined &&
        coord.y !== undefined
      ) {
        return Cartesian3.fromDegrees(coord.x, coord.y, coord.z || 0);
      }
      if (Array.isArray(coord) && coord.length >= 2) {
        return Cartesian3.fromDegrees(coord[0], coord[1], coord[2] || 0);
      }
      return null;
    })
    .filter((p): p is Cartesian3 => p !== null);
}

/**
 * Determine surface type color (floor/roof/wall) for a single polygon.
 * Mirrors the heuristic used in the old processPolygons function.
 */
function getSurfaceTypeColor(polygon: any, globalMinZ: number): Color {
  const exterior: any[] = polygon.exterior || [];
  let minZ = Infinity;
  let maxZ = -Infinity;
  for (const coord of exterior) {
    const z = coord.z || 0;
    if (z < minZ) minZ = z;
    if (z > maxZ) maxZ = z;
  }
  const isFlat = Math.abs(maxZ - minZ) < 0.1;

  if (isFlat && minZ < globalMinZ + 1) {
    return Color.BROWN.withAlpha(0.8); // Floor
  } else if (isFlat) {
    return Color.RED.withAlpha(0.8); // Roof
  } else {
    return Color.BLUE.withAlpha(0.8); // Wall
  }
}

/**
 * Resolve per-polygon color from CityGML material data.
 * Checks polygonMaterials → diffuseColor, falls back to defaultColor.
 * Note: PerInstanceColorAppearance does not support textures; polygonTextures is skipped.
 */
function resolveAppearanceColor(
  globalIndex: number,
  geometry: CityGmlGeometry,
  defaultColor: Color,
): Color {
  const polygonMaterials = geometry.polygonMaterials;
  const materials = geometry.materials;
  if (
    globalIndex >= 0 &&
    Array.isArray(polygonMaterials) &&
    Array.isArray(materials) &&
    materials.length > 0
  ) {
    const matIdx = polygonMaterials[globalIndex];
    if (matIdx != null && materials[matIdx]) {
      const mat = materials[matIdx];
      const [r, g, b] = (mat.diffuseColor as number[]) ?? [1, 1, 1];
      const alpha = 1 - ((mat.transparency as number) ?? 0);
      return new Color(r, g, b, alpha);
    }
  }
  return defaultColor;
}

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

/**
 * Build a GeometryInstance from an already-triangulated TriangularMesh.
 * Vertices are in JGD2011 Zone IX (easting=x, northing=y, height=z in metres)
 * and are converted to WGS84 via proj4 before being stored as ECEF Cartesian3.
 * Per-vertex normals are computed from averaged face normals so that
 * PerInstanceColorAppearance (flat: false) can apply lighting and show terrain relief.
 * Both faces are rendered by setting closed: false on the appearance.
 */
function triangularMeshToGeometryInstance(
  mesh: TriangularMeshData,
  color: Color,
  id: object,
): GeometryInstance | null {
  if (mesh.vertices.length < 3 || mesh.triangles.length === 0) return null;

  // 1) Build ECEF positions (as you already do)
  const positions = new Float64Array(mesh.vertices.length * 3);
  for (let i = 0; i < mesh.vertices.length; i++) {
    const { x, y, z } = mesh.vertices[i];
    const cart = jgd2011Zone9ToCartesian3(x, y, z);
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

  // 2) Compute vertex normals in LOCAL projected space (meters)
  //    so face normals aren't "warped" by ECEF curvature.
  const normalAccumLocal = new Float64Array(mesh.vertices.length * 3);

  for (const tri of mesh.triangles) {
    const i0 = tri[0],
      i1 = tri[1],
      i2 = tri[2];

    const v0 = mesh.vertices[i0];
    const v1 = mesh.vertices[i1];
    const v2 = mesh.vertices[i2];

    const ax = v1.x - v0.x;
    const ay = v1.y - v0.y;
    const az = v1.z - v0.z;

    const bx = v2.x - v0.x;
    const by = v2.y - v0.y;
    const bz = v2.z - v0.z;

    // cross(a,b)
    const nx = ay * bz - az * by;
    const ny = az * bx - ax * bz;
    const nz = ax * by - ay * bx;

    for (const vi of [i0, i1, i2]) {
      normalAccumLocal[vi * 3] += nx;
      normalAccumLocal[vi * 3 + 1] += ny;
      normalAccumLocal[vi * 3 + 2] += nz;
    }
  }

  // 3) Build ONE ENU->ECEF rotation at mesh centroid
  //    (cheap, stable lighting, good for building-scale meshes)
  let cx = 0,
    cy = 0,
    cz = 0;
  for (const v of mesh.vertices) {
    cx += v.x;
    cy += v.y;
    cz += v.z;
  }
  cx /= mesh.vertices.length;
  cy /= mesh.vertices.length;
  cz /= mesh.vertices.length;

  const centerEcef = jgd2011Zone9ToCartesian3(cx, cy, cz);
  const enuFrame = Transforms.eastNorthUpToFixedFrame(centerEcef);
  const rotENUtoECEF = Matrix4.getMatrix3(enuFrame, new Matrix3());

  // 4) Normalize + rotate local normals into ECEF
  const normals = new Float32Array(mesh.vertices.length * 3);
  for (let i = 0; i < mesh.vertices.length; i++) {
    const nxL = normalAccumLocal[i * 3];
    const nyL = normalAccumLocal[i * 3 + 1];
    const nzL = normalAccumLocal[i * 3 + 2];

    const len = Math.sqrt(nxL * nxL + nyL * nyL + nzL * nzL);
    if (len <= 0) continue;

    // local unit normal
    const ux = nxL / len;
    const uy = nyL / len;
    const uz = nzL / len;

    // rotate ENU->ECEF (treat as direction)
    const ex =
      rotENUtoECEF[0] * ux + rotENUtoECEF[3] * uy + rotENUtoECEF[6] * uz;
    const ey =
      rotENUtoECEF[1] * ux + rotENUtoECEF[4] * uy + rotENUtoECEF[7] * uz;
    const ez =
      rotENUtoECEF[2] * ux + rotENUtoECEF[5] * uy + rotENUtoECEF[8] * uz;

    const elen = Math.sqrt(ex * ex + ey * ey + ez * ez);
    if (elen <= 0) continue;

    normals[i * 3] = ex / elen;
    normals[i * 3 + 1] = ey / elen;
    normals[i * 3 + 2] = ez / elen;
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
 * Build a GeometryInstance from a line segment {start, end} in JGD2011 Zone IX.
 * Uses PolylineGeometry with PolylineColorAppearance.VERTEX_FORMAT for
 * per-instance color support.
 */
function lineToGeometryInstance(
  segment: LineSegment,
  color: Color,
  id: object,
): GeometryInstance | null {
  const positions = [
    jgd2011Zone9ToCartesian3(segment.start.x, segment.start.y, segment.start.z),
    jgd2011Zone9ToCartesian3(segment.end.x, segment.end.y, segment.end.z),
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

const MAX_MESH_SAMPLE_POSITIONS = 20;

/**
 * Sample up to MAX_MESH_SAMPLE_POSITIONS evenly-distributed vertices from a
 * TriangularMesh and push Cartesian3 positions into the output array.
 * Used to build the scene-wide BoundingSphere for camera fly-to.
 */
function sampleMeshPositions(
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
    out.push(jgd2011Zone9ToCartesian3(x, y, z));
  }
}

// ── Exported functions ───────────────────────────────────────────────────────

/**
 * Convert a CityGML feature collection into batched Cesium Primitives.
 * All surfaces for all features are packed into at most 2 draw calls:
 * - absolutePrimitive: 3D features (buildings, transport, bridges, etc.)
 * - groundPrimitive:   DEM and unknown/zone features (clamped to ground)
 */
export function convertFeatureCollectionToPrimitives(
  features: CityGmlFeature[],
): PrimitivesResult {
  const absoluteInstances: GeometryInstance[] = [];
  const groundInstances: GeometryInstance[] = [];
  const meshInstances: GeometryInstance[] = [];
  const lineInstances: GeometryInstance[] = [];
  const featureMap = new Map<string, FeatureInstanceData>();
  const sampledPositions: Cartesian3[] = [];

  for (const feature of features) {
    const { geometry, properties } = feature;
    const featureId: string = properties?._originalId || feature.id || "";

    if (!featureId) continue;

    const entry: FeatureInstanceData = {
      feature,
      absoluteInstanceIds: [],
      groundInstanceIds: [],
      meshInstanceIds: [],
      lineInstanceIds: [],
      lodPrimitiveCollection: null,
    };

    // ── Direct TriangularMesh on geometry object ─────────────────────────────
    if (geometry.triangularMesh) {
      const mesh = geometry.triangularMesh as TriangularMeshData;
      const typeConfig = CITYGML_3D_TYPES.find((cfg) => cfg.detect(properties));
      const color = typeConfig?.color ?? Color.CYAN.withAlpha(0.8);
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
      featureMap.set(featureId, entry);
      continue;
    }

    // ── Direct line on geometry object ───────────────────────────────────────
    if (geometry.line) {
      const segment = geometry.line as LineSegment;
      const typeConfig = CITYGML_3D_TYPES.find((cfg) => cfg.detect(properties));
      const color = typeConfig?.color ?? Color.YELLOW.withAlpha(0.9);
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
            jgd2011Zone9ToCartesian3(
              segment.start.x,
              segment.start.y,
              segment.start.z,
            ),
          );
        }
      }
      featureMap.set(featureId, entry);
      continue;
    }

    const gmlGeometries =
      geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
    if (!gmlGeometries || !Array.isArray(gmlGeometries)) continue;

    // ── DEM ─────────────────────────────────────────────────────────────────
    if (isDemFeature(properties)) {
      const allPolygons: any[] = [];
      for (const geom of gmlGeometries) {
        if (Array.isArray(geom.polygons)) allPolygons.push(...geom.polygons);
      }

      const step = Math.max(
        1,
        Math.ceil(allPolygons.length / MAX_DEM_POLYGONS),
      );

      allPolygons
        .filter((_, i) => i % step === 0)
        .forEach((polygon, idx) => {
          if (!polygon.exterior || !Array.isArray(polygon.exterior)) return;
          const positions = coordsToPositions(polygon.exterior);
          if (positions.length < 3) return;

          const instanceId: InstanceId = {
            _originalId: featureId,
            featureId,
            instanceId: `${featureId}_dem_${idx}`,
          };

          groundInstances.push(
            new GeometryInstance({
              id: instanceId,
              geometry: new PolygonGeometry({
                polygonHierarchy: new PolygonHierarchy(positions),
                perPositionHeight: false,
              }),
              attributes: {
                color: ColorGeometryInstanceAttribute.fromColor(
                  Color.SANDYBROWN,
                ),
                show: new ShowGeometryInstanceAttribute(true),
              },
            }),
          );
          entry.groundInstanceIds.push(instanceId);

          if (sampledPositions.length < 500)
            sampledPositions.push(positions[0]);
        });
    }
    // ── Known 3D type ────────────────────────────────────────────────────────
    else if (CITYGML_3D_TYPES.some((cfg) => cfg.detect(properties))) {
      const typeConfig = CITYGML_3D_TYPES.find((cfg) => cfg.detect(properties));
      if (!typeConfig) {
        featureMap.set(featureId, entry);
        continue;
      }

      // LOD selection: prefer LOD1 → LOD2 → LOD3 → any
      let selectedGeometries = gmlGeometries.filter((g: any) => g.lod === 1);
      if (selectedGeometries.length === 0) {
        selectedGeometries = gmlGeometries.filter(
          (g: any) =>
            g.lod === 2 ||
            g.gml_trait?.property?.includes("Lod2") ||
            g.gml_trait?.property?.includes("LOD2"),
        );
      }
      if (selectedGeometries.length === 0) {
        selectedGeometries = gmlGeometries.filter(
          (g: any) =>
            g.lod === 3 ||
            g.gml_trait?.property?.includes("Lod3") ||
            g.gml_trait?.property?.includes("LOD3"),
        );
      }
      if (selectedGeometries.length === 0) selectedGeometries = gmlGeometries;

      // ── TriangularMesh path ───────────────────────────────────────────────
      const meshGeometries = selectedGeometries.filter(
        (g: any) => g.triangularMesh,
      );
      if (meshGeometries.length > 0) {
        for (const geom of meshGeometries) {
          const mesh = geom.triangularMesh as TriangularMeshData;
          const instanceId: InstanceId = {
            _originalId: featureId,
            featureId,
            instanceId: `${featureId}_mesh_0`,
          };
          const instance = triangularMeshToGeometryInstance(
            mesh,
            typeConfig.color,
            instanceId,
          );
          if (instance) {
            meshInstances.push(instance);
            entry.meshInstanceIds.push(instanceId);
            sampleMeshPositions(mesh, sampledPositions, 500);
          }
        }
        featureMap.set(featureId, entry);
        continue;
      }

      const allPolygons: any[] = [];
      for (const geom of selectedGeometries) {
        if (geom.polygons && Array.isArray(geom.polygons)) {
          const baseIndex: number = geom.pos ?? 0;
          geom.polygons.forEach((polygon: any, localIdx: number) => {
            allPolygons.push({
              ...polygon,
              _globalIndex: baseIndex + localIdx,
            });
          });
        }
      }

      if (allPolygons.length === 0) {
        featureMap.set(featureId, entry);
        continue;
      }

      // Compute global ground level for height normalization
      let globalMinZ = Infinity;
      for (const p of allPolygons) {
        for (const c of p.exterior || []) {
          const z = c.z || 0;
          if (z < globalMinZ) globalMinZ = z;
        }
      }

      if (globalMinZ === Infinity) {
        featureMap.set(featureId, entry);
        continue;
      }

      allPolygons.forEach((polygon, idx) => {
        if (!polygon.exterior || !Array.isArray(polygon.exterior)) return;
        const rawPositions = coordsToPositions(polygon.exterior);
        if (rawPositions.length < 3) return;

        // Normalize height: shift ground level to 0
        const positions = rawPositions.map((pos) => {
          const carto = Cartographic.fromCartesian(pos);
          carto.height = (carto.height || 0) - globalMinZ;
          return Cartographic.toCartesian(carto);
        });

        const globalIdx: number = polygon._globalIndex ?? -1;
        const defaultColor = typeConfig.useSurfaceTypeColors
          ? getSurfaceTypeColor(polygon, globalMinZ)
          : typeConfig.color;
        const color =
          typeConfig.displayName === "Building"
            ? resolveAppearanceColor(globalIdx, geometry, defaultColor)
            : defaultColor;

        const instanceId: InstanceId = {
          _originalId: featureId,
          featureId,
          instanceId: `${featureId}_abs_${idx}`,
        };

        absoluteInstances.push(
          new GeometryInstance({
            id: instanceId,
            geometry: new PolygonGeometry({
              polygonHierarchy: new PolygonHierarchy(positions),
              perPositionHeight: true,
            }),
            attributes: {
              color: ColorGeometryInstanceAttribute.fromColor(color),
              show: new ShowGeometryInstanceAttribute(true),
            },
          }),
        );
        entry.absoluteInstanceIds.push(instanceId);

        if (sampledPositions.length < 500) sampledPositions.push(positions[0]);
      });
    }
    // ── Unknown type → ground with CYAN ─────────────────────────────────────
    else {
      // TriangularMesh in unknown-type features → mesh primitive with CYAN
      const unknownMeshGeoms = gmlGeometries.filter(
        (g: any) => g.triangularMesh,
      );
      if (unknownMeshGeoms.length > 0) {
        for (const geom of unknownMeshGeoms) {
          const mesh = geom.triangularMesh as TriangularMeshData;
          const instanceId: InstanceId = {
            _originalId: featureId,
            featureId,
            instanceId: `${featureId}_mesh_unknown_0`,
          };
          const instance = triangularMeshToGeometryInstance(
            mesh,
            Color.CYAN.withAlpha(0.8),
            instanceId,
          );
          if (instance) {
            meshInstances.push(instance);
            entry.meshInstanceIds.push(instanceId);
            sampleMeshPositions(mesh, sampledPositions, 500);
          }
        }
        featureMap.set(featureId, entry);
        continue;
      }

      const allPolygons: any[] = [];
      for (const geom of gmlGeometries) {
        if (Array.isArray(geom.polygons)) allPolygons.push(...geom.polygons);
      }

      allPolygons.forEach((polygon, idx) => {
        if (!polygon.exterior || !Array.isArray(polygon.exterior)) return;
        const positions = coordsToPositions(polygon.exterior);
        if (positions.length < 3) return;

        const instanceId: InstanceId = {
          _originalId: featureId,
          featureId,
          instanceId: `${featureId}_ground_${idx}`,
        };

        groundInstances.push(
          new GeometryInstance({
            id: instanceId,
            geometry: new PolygonGeometry({
              polygonHierarchy: new PolygonHierarchy(positions),
              perPositionHeight: false,
            }),
            attributes: {
              color: ColorGeometryInstanceAttribute.fromColor(Color.CYAN),
              show: new ShowGeometryInstanceAttribute(true),
            },
          }),
        );
        entry.groundInstanceIds.push(instanceId);

        if (sampledPositions.length < 500) sampledPositions.push(positions[0]);
      });
    }

    featureMap.set(featureId, entry);
  }

  const absolutePrimitive =
    absoluteInstances.length > 0
      ? new Primitive({
          geometryInstances: absoluteInstances,
          appearance: new PerInstanceColorAppearance({
            translucent: true,
            flat: false,
          }),
          asynchronous: true,
        })
      : null;

  const groundPrimitive =
    groundInstances.length > 0
      ? new GroundPrimitive({
          geometryInstances: groundInstances,
          appearance: new PerInstanceColorAppearance({
            translucent: false,
            flat: true,
          }),
          asynchronous: true,
        })
      : null;

  // asynchronous: false — custom Geometry has no _workerName, so must be synchronous
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

  // PolylineColorAppearance supports per-instance color via ColorGeometryInstanceAttribute
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

  return {
    absolutePrimitive,
    groundPrimitive,
    meshPrimitive,
    linePrimitive,
    featureMap,
    boundingSphere,
  };
}

/**
 * Create a LOD-upgrade Primitive Collection for a single feature using LOD3 (fallback LOD2) geometry.
 * Returns null if no higher-LOD data is available.
 */
export function createLodUpgradePrimitiveCollection(
  feature: CityGmlFeature,
  typeConfig?: CityGmlTypeConfig,
): PrimitiveCollection | null {
  const { geometry } = feature;
  const featureId: string = feature.properties?._originalId || feature.id || "";

  const gmlGeometries =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
  if (!gmlGeometries || !Array.isArray(gmlGeometries)) return null;

  // LOD3 first, fallback to LOD2
  let lodGeometries = gmlGeometries.filter(
    (geom: any) =>
      geom.lod === 3 ||
      geom.gml_trait?.property?.includes("Lod3") ||
      geom.gml_trait?.property?.includes("LOD3"),
  );
  if (lodGeometries.length === 0) {
    lodGeometries = gmlGeometries.filter(
      (geom: any) =>
        geom.lod === 2 ||
        geom.gml_trait?.property?.includes("Lod2") ||
        geom.gml_trait?.property?.includes("LOD2"),
    );
  }

  if (lodGeometries.length === 0) return null;

  // ── TriangularMesh LOD path ──────────────────────────────────────────────
  const meshGeometries = lodGeometries.filter((g: any) => g.triangularMesh);
  if (meshGeometries.length > 0) {
    const fillInstances: GeometryInstance[] = [];
    for (const geom of meshGeometries) {
      const mesh = geom.triangularMesh as TriangularMeshData;
      const defaultColor = typeConfig?.color ?? Color.GRAY.withAlpha(0.8);
      const instance = triangularMeshToGeometryInstance(mesh, defaultColor, {
        featureId,
        instanceId: `${featureId}_fill_lod_mesh_0`,
      });
      if (instance) fillInstances.push(instance);
    }
    if (fillInstances.length === 0) return null;
    const meshPrimitive = new Primitive({
      geometryInstances: fillInstances,
      appearance: new PerInstanceColorAppearance({
        flat: true,
        translucent: true,
        closed: false,
      }),
      asynchronous: false,
    });
    const col = new PrimitiveCollection();
    col.add(meshPrimitive);
    return col;
  }

  const allPolygons: any[] = [];
  for (const geom of lodGeometries) {
    if (geom.polygons && Array.isArray(geom.polygons)) {
      const baseIndex: number = geom.pos ?? 0;
      geom.polygons.forEach((polygon: any, localIdx: number) => {
        allPolygons.push({ ...polygon, _globalIndex: baseIndex + localIdx });
      });
    }
  }

  if (allPolygons.length === 0) return null;

  let globalMinZ = Infinity;
  for (const p of allPolygons) {
    for (const c of p.exterior || []) {
      const z = c.z || 0;
      if (z < globalMinZ) globalMinZ = z;
    }
  }

  if (globalMinZ === Infinity) return null;

  const fillInstances: GeometryInstance[] = [];
  const outlineInstances: GeometryInstance[] = [];

  allPolygons.forEach((polygon, idx) => {
    if (!polygon.exterior || !Array.isArray(polygon.exterior)) return;
    const rawPositions = coordsToPositions(polygon.exterior);
    if (rawPositions.length < 3) return;

    const positions = rawPositions.map((pos) => {
      const carto = Cartographic.fromCartesian(pos);
      carto.height = (carto.height || 0) - globalMinZ;
      return Cartographic.toCartesian(carto);
    });

    const globalIdx: number = polygon._globalIndex ?? -1;
    const defaultColor = typeConfig?.useSurfaceTypeColors
      ? getSurfaceTypeColor(polygon, globalMinZ)
      : (typeConfig?.color ?? Color.GRAY.withAlpha(0.8));
    const color = resolveAppearanceColor(globalIdx, geometry, defaultColor);

    fillInstances.push(
      new GeometryInstance({
        id: { featureId, instanceId: `${featureId}_fill_lod_${idx}` },
        geometry: new PolygonGeometry({
          polygonHierarchy: new PolygonHierarchy(positions),
          perPositionHeight: true,
        }),
        attributes: {
          color: ColorGeometryInstanceAttribute.fromColor(color),
          show: new ShowGeometryInstanceAttribute(true),
        },
      }),
    );

    outlineInstances.push(
      new GeometryInstance({
        id: { featureId, instanceId: `${featureId}_outline_lod_${idx}` },
        geometry: new PolygonOutlineGeometry({
          polygonHierarchy: new PolygonHierarchy(positions),
          perPositionHeight: true,
        }),
        attributes: {
          color: ColorGeometryInstanceAttribute.fromColor(
            Color.BLACK.withAlpha(0.8),
          ),
          show: new ShowGeometryInstanceAttribute(true),
        },
      }),
    );
  });

  if (fillInstances.length === 0 || outlineInstances.length === 0) return null;
  const fillPrimitive = new Primitive({
    geometryInstances: fillInstances,
    appearance: new PerInstanceColorAppearance({
      flat: true,
      translucent: true,
    }),
  });

  const outlinePrimitive = new Primitive({
    geometryInstances: outlineInstances,
    appearance: new PerInstanceColorAppearance({
      flat: true,
      translucent: true,
    }),
  });

  const primitiveCollection = new PrimitiveCollection();
  primitiveCollection.add(fillPrimitive);
  primitiveCollection.add(outlinePrimitive);
  return primitiveCollection;
}
