import {
  BoundingSphere,
  Cartesian3,
  Cartographic,
  Color,
  ColorGeometryInstanceAttribute,
  GeometryInstance,
  GroundPrimitive,
  PerInstanceColorAppearance,
  PolygonGeometry,
  PolygonHierarchy,
  PolygonOutlineGeometry,
  Primitive,
  PrimitiveCollection,
  ShowGeometryInstanceAttribute,
} from "cesium";

// ── Types ────────────────────────────────────────────────────────────────────

/**
 * The geometry the batched renderer reads.
 *
 * `type` is `"CityGmlGeometry"` from the legacy transform and `"MultiPolygon"`
 * from the new-geometry one, which keeps the GeoJSON projection for the table
 * and attaches the levels of detail alongside it. `gmlGeometries` is the field
 * that matters here, and both transforms write it.
 */
type CityGmlGeometry = {
  type: string;
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
  lodPrimitiveCollection: PrimitiveCollection | null;
};

export type PrimitivesResult = {
  absolutePrimitive: Primitive | null;
  groundPrimitive: GroundPrimitive | null;
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
        p?.["__citygml_feature_type"]?.includes("tran:") ||
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
        p?.["__citygml_feature_type"]?.includes("brid:") ||
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
        p?.["__citygml_feature_type"]?.includes("frn:") ||
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
        p?.["__citygml_feature_type"]?.includes("veg:") ||
        p?.["veg:class"] ||
        p?.cityGmlAttributes?.["veg:class"]
      ),
  },
];

// ── Internal helpers ─────────────────────────────────────────────────────────

const MAX_DEM_POLYGONS = 150;

function isDemFeature(properties: Record<string, any>): boolean {
  return !!(
    properties?.gmlName?.includes("dem:") ||
    properties?.featureType?.includes("dem:") ||
    properties?.["__citygml_feature_type"]?.includes("dem:") ||
    properties?.["dem:class"] ||
    properties?.["dem:type"] ||
    properties?.cityGmlAttributes?.["dem:class"] ||
    properties?.cityGmlAttributes?.["dem:type"]
  );
}

/**
 * One coordinate as `[longitude, latitude, height]`, or null if unreadable.
 *
 * The legacy transform passes the engine's `{ x, y, z }` records through; the
 * new-geometry one emits GeoJSON positions, `[lon, lat, z]`. Everything that
 * reads a coordinate has to accept both.
 */
export function readCoord(coord: any): [number, number, number] | null {
  if (Array.isArray(coord)) {
    return coord.length >= 2 ? [coord[0], coord[1], coord[2] ?? 0] : null;
  }
  if (typeof coord?.x === "number" && typeof coord?.y === "number") {
    return [coord.x, coord.y, coord.z ?? 0];
  }
  return null;
}

/**
 * Height from either coordinate form.
 *
 * Read on its own by the surface-type heuristic and the ground-level scan; on
 * a new-format file a `coord.z` there returns 0 for every position, which
 * flattens every building and colours the lot as floor.
 */
export function coordZ(coord: any): number {
  return readCoord(coord)?.[2] ?? 0;
}

/**
 * Whether a feature's geometry is CityGML, in either format.
 *
 * Legacy says so in its type. The new-geometry transform emits a MultiPolygon
 * like anything else and marks it with the level of detail it chose — which it
 * writes only for a collection whose members declared one, and only the
 * CityGML pipeline does that.
 *
 * Shared, because getting this wrong in one place is invisible: the split in
 * `Cesium/index.tsx` decides whether a feature is drawn as batched primitives
 * or as one entity per surface, and the branch in the debug panel's
 * `handleFlyToSelectedFeature` decides whether the camera can find it at all.
 */
export function isCityGmlGeometry(geometry: any): boolean {
  return (
    geometry?.type === "CityGmlGeometry" || typeof geometry?.lod === "number"
  );
}

/**
 * The rings of each surface, when the geometry is a GeoJSON polygon form.
 *
 * `coordinates` is already one entry per surface, each `[exterior, ...holes]`.
 */
function polygonRingsOf(geometry: CityGmlGeometry): any[][] | null {
  const { type, coordinates } = geometry;
  if (!Array.isArray(coordinates)) return null;
  if (type === "MultiPolygon") return coordinates;
  if (type === "Polygon") return [coordinates];
  return null;
}

/**
 * The per-level surface lists to draw from.
 *
 * The legacy transform hands over the engine's own `gmlGeometries`, grouped by
 * level of detail and indexed into the appearance arrays by `pos`. The
 * new-geometry transform has already chosen a level and flattened it to a
 * MultiPolygon of one polygon per surface, so its rings are read straight off
 * `coordinates`.
 *
 * Reading them rather than having the transform emit a second copy is the
 * whole point: a CityGML file is mostly coordinates, and holding them twice —
 * or holding every level when only one is drawn — is what exhausts the tab.
 * The wrappers built here live only for this call.
 */
export function gmlGeometriesOf(geometry: CityGmlGeometry): any[] | null {
  const stored =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
  if (Array.isArray(stored)) return stored;

  const rings = polygonRingsOf(geometry);
  if (!rings) return null;

  // Holes are dropped, as they are on the legacy path — `PolygonHierarchy` is
  // built from the outer ring alone below.
  return [
    {
      lod: typeof geometry.lod === "number" ? geometry.lod : undefined,
      pos: 0,
      polygons: rings.map((surface) => ({ exterior: surface[0] })),
    },
  ];
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
    const z = coordZ(coord);
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
  const featureMap = new Map<string, FeatureInstanceData>();
  const sampledPositions: Cartesian3[] = [];

  for (const feature of features) {
    const { geometry, properties } = feature;
    const featureId: string = properties?._originalId || feature.id || "";

    if (!featureId) continue;

    const gmlGeometries = gmlGeometriesOf(geometry);
    if (!gmlGeometries) continue;

    const entry: FeatureInstanceData = {
      feature,
      absoluteInstanceIds: [],
      groundInstanceIds: [],
      lodPrimitiveCollection: null,
    };

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
          const z = coordZ(c);
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

  const boundingSphere =
    sampledPositions.length > 0
      ? BoundingSphere.fromPoints(sampledPositions)
      : null;

  return { absolutePrimitive, groundPrimitive, featureMap, boundingSphere };
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
  // Only the legacy path reaches here with something to upgrade to: the
  // new-geometry transform keeps one level, so there is no finer one held.
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
      const z = coordZ(c);
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
