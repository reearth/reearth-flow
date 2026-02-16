import {
  Entity,
  Color,
  Cartesian2,
  Cartesian3,
  HeightReference,
  ColorMaterialProperty,
  PolygonHierarchy,
  ImageMaterialProperty,
  CheckerboardMaterialProperty,
  ConstantProperty,
  PropertyBag,
  PolygonGraphics,
  PointGraphics,
  PolylineGraphics,
  ConstantPositionProperty,
  Cartographic,
  Viewer,
} from "cesium";

import { generateUUID } from "@flow/utils";

// Extend Entity type to include child surface entities and LOD upgrade entities
export type EntityWithSurfaces = Entity & {
  surfaces?: Entity[];
  lodSurfaces?: Entity[];
};

type ProcessedPolygon = {
  positions: Cartesian3[];
  surfaceType: string;
  material: ColorMaterialProperty;
  minZ: number;
  maxZ: number;
};

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

/**
 * Convert a CityGML feature to a Cesium Entity
 * This handles various CityGML geometry types and building-specific properties
 */
export function convertFeatureToEntity(
  feature: CityGmlFeature,
): EntityWithSurfaces | null {
  const { geometry, properties, id } = feature;

  if (!geometry || geometry.type !== "CityGmlGeometry") {
    return null;
  }

  const entity = new Entity({
    id: id || generateUUID(),
    name:
      properties?.name || extractBuildingName(geometry) || "CityGML Feature",
  });

  // Try config-driven 3D converter first (handles bldg, tran, brid, frn, veg)
  if (convertGeneric3DGeometry(entity, geometry, properties)) {
    return entity;
  }

  if (convertCityGmlSurfaceGeometry(entity, geometry, properties)) {
    return entity;
  }

  if (convertSurfaceGeometry(entity, geometry, properties)) {
    return entity;
  }

  if (convertGenericGeometry(entity, geometry, properties)) {
    return entity;
  }

  // Fallback: Create a simple point entity for any CityGML data
  if (createFallbackEntity(entity, geometry, properties)) {
    return entity;
  }

  return null;
}

/**
 * Process raw CityGML building polygons into positions arrays with surface metadata.
 * Handles Z-analysis, surface type detection, ground level normalization, and coordinate conversion.
 */
function processPolygons(buildingPolygons: any[]): ProcessedPolygon[] {
  const results: ProcessedPolygon[] = [];

  // Pre-compute global ground level across all polygons (loop to avoid stack overflow on large LOD3 data)
  let globalMinZ = Infinity;
  for (const p of buildingPolygons) {
    for (const c of p.exterior || []) {
      const z = c.z || 0;
      if (z < globalMinZ) globalMinZ = z;
    }
  }
  if (globalMinZ === Infinity) return results;

  buildingPolygons.forEach((polygon) => {
    if (!polygon.exterior || !Array.isArray(polygon.exterior)) return;

    let minZ = Infinity;
    let maxZ = -Infinity;
    for (const coord of polygon.exterior) {
      const z = coord.z || 0;
      if (z < minZ) minZ = z;
      if (z > maxZ) maxZ = z;
    }
    const isFlat = Math.abs(maxZ - minZ) < 0.1;

    let surfaceType: string;
    let material: ColorMaterialProperty;

    if (isFlat && minZ < globalMinZ + 1) {
      surfaceType = "Floor";
      material = new ColorMaterialProperty(Color.BROWN.withAlpha(0.8));
    } else if (isFlat) {
      surfaceType = "Roof";
      material = new ColorMaterialProperty(Color.RED.withAlpha(0.8));
    } else {
      surfaceType = "Wall";
      material = new ColorMaterialProperty(Color.BLUE.withAlpha(0.8));
    }

    const rawPositions = convertCoordinatesToPositions(polygon.exterior);
    const positions = rawPositions.map((pos) => {
      const cartographic = Cartographic.fromCartesian(pos);
      cartographic.height = (cartographic.height || 0) - globalMinZ;
      return Cartographic.toCartesian(cartographic);
    });

    if (positions.length >= 3) {
      results.push({ positions, surfaceType, material, minZ, maxZ });
    }
  });

  return results;
}

// ── Generic 3D CityGML feature type config ──────────────────────────────────

type CityGmlTypeConfig = {
  /** Property prefixes / keywords used to detect this type */
  detect: (props: Record<string, any>) => boolean;
  /** Human-readable display name */
  displayName: string;
  /** Material color for surfaces */
  color: Color;
  /** When true, use processPolygons surface-type colors (wall=blue, roof=red, floor=brown) instead of uniform color */
  useSurfaceTypeColors?: boolean;
  /** Attribute keys to extract for InfoBox (prefix:key) */
  attrKeys: string[];
};

const CITYGML_3D_TYPES: CityGmlTypeConfig[] = [
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

/**
 * Generic 3D CityGML converter for brid, frn, veg (and future types).
 * Config-driven: add entries to CITYGML_3D_TYPES to support new feature types.
 */
function convertGeneric3DGeometry(
  entity: EntityWithSurfaces,
  geometry: CityGmlGeometry,
  properties: Record<string, any>,
): boolean {
  const gmlGeometries =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;

  if (!gmlGeometries || !Array.isArray(gmlGeometries)) return false;

  // Find matching type config
  const typeConfig = CITYGML_3D_TYPES.find((cfg) => cfg.detect(properties));
  if (!typeConfig) return false;

  // LOD1 on initial load for performance; higher LODs available on selection
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
  if (selectedGeometries.length === 0) {
    selectedGeometries = gmlGeometries;
  }

  const allPolygons: any[] = [];
  selectedGeometries.forEach((geom: any) => {
    if (geom.polygons && Array.isArray(geom.polygons)) {
      allPolygons.push(...geom.polygons);
    }
  });
  if (allPolygons.length === 0) return false;

  const processed = processPolygons(allPolygons);
  if (processed.length === 0) return false;

  const surfaces: Entity[] = [];
  entity.surfaces = surfaces;

  processed.forEach((p, index) => {
    const material = typeConfig.useSurfaceTypeColors
      ? p.material
      : new ColorMaterialProperty(typeConfig.color);

    const surfaceEntity = new Entity({
      id: `${entity.id}_${typeConfig.displayName.toLowerCase().replace(/\s/g, "")}_${index}`,
      name: `${entity.name} - ${p.surfaceType} ${index + 1}`,
      polygon: new PolygonGraphics({
        hierarchy: new ConstantProperty(new PolygonHierarchy(p.positions)),
        material,
        outline: new ConstantProperty(true),
        outlineColor: new ConstantProperty(Color.BLACK.withAlpha(0.6)),
        outlineWidth: new ConstantProperty(1),
        heightReference: new ConstantProperty(HeightReference.NONE),
        perPositionHeight: new ConstantProperty(true),
      }),
    });

    const surfacePropertyBag = new PropertyBag(properties);
    surfacePropertyBag.addProperty("cityGmlType", typeConfig.displayName);
    surfacePropertyBag.addProperty("surfaceType", p.surfaceType);
    surfaceEntity.properties = surfacePropertyBag;

    surfaces.push(surfaceEntity);
  });

  // Build InfoBox from config attrKeys
  const featureId = properties?.gmlId || properties?.id || "Unknown";
  const featureType =
    properties?.gmlName ||
    properties?.featureType ||
    properties?.metadata?.featureType ||
    typeConfig.displayName;

  const attrRows = typeConfig.attrKeys
    .map((key) => {
      const value = properties?.cityGmlAttributes?.[key] || properties?.[key];
      if (!value) return "";
      const label = key.split(":").pop() || key;
      return `<tr style="border-bottom: 1px solid #eee;">
        <td style="padding: 4px 8px 4px 0; font-weight: bold;">${label}:</td>
        <td style="padding: 4px 0;">${value}</td>
      </tr>`;
    })
    .filter(Boolean)
    .join("");

  const infoBoxHtml = `
    <div style="font-family: sans-serif; line-height: 1.4;">
      <h3 style="margin: 0 0 10px 0; color: #2c3e50;">CityGML ${typeConfig.displayName}</h3>
      <table style="width: 100%; border-collapse: collapse;">
        <tr style="border-bottom: 1px solid #eee;">
          <td style="padding: 4px 8px 4px 0; font-weight: bold; width: 40%;">ID:</td>
          <td style="padding: 4px 0;">${featureId}</td>
        </tr>
        <tr style="border-bottom: 1px solid #eee;">
          <td style="padding: 4px 8px 4px 0; font-weight: bold;">Type:</td>
          <td style="padding: 4px 0;">${featureType}</td>
        </tr>
        <tr style="border-bottom: 1px solid #eee;">
          <td style="padding: 4px 8px 4px 0; font-weight: bold;">Surfaces:</td>
          <td style="padding: 4px 0;">${processed.length}</td>
        </tr>
        ${attrRows}
      </table>
    </div>`;

  entity.description = new ConstantProperty(infoBoxHtml);

  // Center position from first polygon
  const positionsForCenter = processed[0].positions;
  if (positionsForCenter.length > 0) {
    let totalX = 0,
      totalY = 0,
      totalZ = 0;
    positionsForCenter.forEach((pos: Cartesian3) => {
      totalX += pos.x;
      totalY += pos.y;
      totalZ += pos.z;
    });
    entity.position = new ConstantPositionProperty(
      new Cartesian3(
        totalX / positionsForCenter.length,
        totalY / positionsForCenter.length,
        totalZ / positionsForCenter.length,
      ),
    );
  }

  const propertyBag = new PropertyBag(properties);
  propertyBag.addProperty("cityGmlType", typeConfig.displayName);
  propertyBag.addProperty("surfaceCount", surfaces.length);
  entity.properties = propertyBag;

  return true;
}

/**
 * Convert non-building CityGML geometry (Zones, LandUse, Relief, etc.)
 * Handles gmlGeometries structure for any CityGML type that isn't a building
 * Creates a single merged polygon entity instead of separate surfaces
 */
function convertCityGmlSurfaceGeometry(
  entity: Entity,
  geometry: CityGmlGeometry,
  properties: Record<string, any>,
): boolean {
  // Access the CityGML geometry structure
  const gmlGeometries =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;

  if (!gmlGeometries || !Array.isArray(gmlGeometries)) {
    return false;
  }

  // Skip types handled by convertGeneric3DGeometry (bldg, tran, brid, frn, veg)
  if (CITYGML_3D_TYPES.some((cfg) => cfg.detect(properties))) {
    return false;
  }

  // Determine feature type for styling
  const featureType =
    properties?.gmlName ||
    properties?.featureType ||
    properties?.metadata?.featureType ||
    "CityGML Feature";

  // Extract feature information for InfoBox
  const featureId =
    properties?.gmlId || properties?.id || properties?.metadata?.featureId;
  const lodLevel = properties?.metadata?.lod;

  // Collect all polygons to merge into one entity
  const allPolygonHierarchies: PolygonHierarchy[] = [];
  let totalVertices = 0;

  gmlGeometries.forEach((geom: any) => {
    if (geom.polygons && Array.isArray(geom.polygons)) {
      geom.polygons.forEach((polygon: any) => {
        if (polygon.exterior && Array.isArray(polygon.exterior)) {
          const positions = convertCoordinatesToPositions(polygon.exterior);
          if (positions.length >= 3) {
            allPolygonHierarchies.push(new PolygonHierarchy(positions));
            totalVertices += positions.length;
          }
        }
      });
    }
  });

  if (allPolygonHierarchies.length === 0) {
    return false;
  }

  // Determine material based on feature type
  let material: ColorMaterialProperty;
  let displayName: string;

  if (featureType.includes("Zone") || featureType.includes("urf:")) {
    material = new ColorMaterialProperty(Color.YELLOW.withAlpha(0.6));
    displayName = "Zone";
  } else if (featureType.includes("LandUse")) {
    material = new ColorMaterialProperty(Color.GREEN.withAlpha(0.6));
    displayName = "Land Use";
  } else if (featureType.includes("Relief")) {
    material = new ColorMaterialProperty(Color.BROWN.withAlpha(0.6));
    displayName = "Relief";
  } else if (featureType.includes("WaterBody")) {
    material = new ColorMaterialProperty(Color.BLUE.withAlpha(0.6));
    displayName = "Water Body";
  } else {
    material = new ColorMaterialProperty(Color.CYAN.withAlpha(0.6));
    displayName = "Surface";
  }

  // Create a single polygon entity with all the polygons
  // For flat ground-level features, use height = 0 and CLAMP_TO_GROUND
  entity.polygon = new PolygonGraphics({
    hierarchy: new ConstantProperty(allPolygonHierarchies[0]),
    material: material,
    outline: new ConstantProperty(false), // Disable outline for terrain-clamped polygons
    outlineColor: new ConstantProperty(Color.BLACK.withAlpha(0.8)),
    outlineWidth: new ConstantProperty(2),
    height: new ConstantProperty(0), // Required for heightReference to work
    heightReference: new ConstantProperty(HeightReference.CLAMP_TO_GROUND),
  });

  // If there are multiple polygons, store them as additional entities
  if (allPolygonHierarchies.length > 1) {
    const additionalEntities: Entity[] = [];
    for (let i = 1; i < allPolygonHierarchies.length; i++) {
      const additionalEntity = new Entity({
        id: `${entity.id}_polygon_${i}`,
        polygon: new PolygonGraphics({
          hierarchy: new ConstantProperty(allPolygonHierarchies[i]),
          material: material,
          outline: new ConstantProperty(false), // Disable outline for terrain-clamped polygons
          outlineColor: new ConstantProperty(Color.BLACK.withAlpha(0.8)),
          outlineWidth: new ConstantProperty(2),
          height: new ConstantProperty(0), // Required for heightReference to work
          heightReference: new ConstantProperty(
            HeightReference.CLAMP_TO_GROUND,
          ),
        }),
      });
      additionalEntities.push(additionalEntity);
    }
    // Store additional entities on the main entity for rendering
    (entity as any).additionalPolygons = additionalEntities;
  }

  // Create InfoBox content
  const infoBoxHtml = `
  <div style="font-family: sans-serif; line-height: 1.4;">
    <h3 style="margin: 0 0 10px 0; color: #2c3e50;">📍 ${featureType}</h3>

    <table style="width: 100%; border-collapse: collapse;">
      ${
        featureId
          ? `<tr style="border-bottom: 1px solid #eee;">
        <td style="padding: 4px 8px 4px 0; font-weight: bold; width: 40%;">ID:</td>
        <td style="padding: 4px 0;">${featureId}</td>
      </tr>`
          : ""
      }
      <tr style="border-bottom: 1px solid #eee;">
        <td style="padding: 4px 8px 4px 0; font-weight: bold;">Type:</td>
        <td style="padding: 4px 0;">${featureType}</td>
      </tr>
      ${
        lodLevel
          ? `<tr style="border-bottom: 1px solid #eee;">
        <td style="padding: 4px 8px 4px 0; font-weight: bold;">LOD:</td>
        <td style="padding: 4px 0;">${lodLevel}</td>
      </tr>`
          : ""
      }
      <tr style="border-bottom: 1px solid #eee;">
        <td style="padding: 4px 8px 4px 0; font-weight: bold;">Polygons:</td>
        <td style="padding: 4px 0;">${allPolygonHierarchies.length}</td>
      </tr>
      <tr>
        <td style="padding: 4px 8px 4px 0; font-weight: bold;">Total Vertices:</td>
        <td style="padding: 4px 0;">${totalVertices}</td>
      </tr>
    </table>

    ${
      Object.keys(properties?.cityGmlAttributes || {}).length > 0
        ? `
      <h4 style="margin: 10px 0 5px 0; color: #2c3e50;">Attributes</h4>
      <table style="width: 100%; border-collapse: collapse; font-size: 12px;">
        ${Object.entries(properties.cityGmlAttributes)
          .map(
            ([key, value]) => `
          <tr style="border-bottom: 1px solid #eee;">
            <td style="padding: 2px 8px 2px 0; font-weight: bold;">${key}:</td>
            <td style="padding: 2px 0;">${value}</td>
          </tr>
        `,
          )
          .join("")}
      </table>
    `
        : ""
    }
  </div>`;

  entity.description = new ConstantProperty(infoBoxHtml);

  // Store properties on the entity
  const propertyBag = new PropertyBag(properties);
  propertyBag.addProperty("cityGmlType", displayName);
  propertyBag.addProperty("featureType", featureType);
  propertyBag.addProperty("polygonCount", allPolygonHierarchies.length);
  propertyBag.addProperty("totalVertices", totalVertices);
  entity.properties = propertyBag;

  return true;
}

/**
 * Convert surface geometry (walls, roofs, floors, etc.)
 */
function convertSurfaceGeometry(
  entity: Entity,
  geometry: CityGmlGeometry,
  properties: Record<string, any>,
): boolean {
  const surfaces = geometry.surfaces || geometry.boundedBy;
  const coordinates = geometry.coordinates;

  if (!surfaces && !coordinates) {
    return false;
  }

  if (coordinates && Array.isArray(coordinates)) {
    // Convert coordinate arrays to Cesium positions
    const positions = convertCoordinatesToPositions(coordinates);

    if (positions.length >= 3) {
      entity.polygon = new PolygonGraphics({
        hierarchy: new ConstantProperty(new PolygonHierarchy(positions)),
        material: createSurfaceMaterial(geometry, properties),
        heightReference: new ConstantProperty(HeightReference.CLAMP_TO_GROUND),
        outline: new ConstantProperty(true),
        outlineColor: new ConstantProperty(Color.WHITE.withAlpha(0.8)),
      });

      const propertyBag = new PropertyBag(properties);
      propertyBag.addProperty("cityGmlType", "Surface");
      propertyBag.addProperty(
        "surfaceType",
        geometry.surfaceType || properties?.type,
      );
      entity.properties = propertyBag;

      return true;
    }
  }

  return false;
}

/**
 * Convert generic CityGML geometry types
 */
function convertGenericGeometry(
  entity: Entity,
  geometry: CityGmlGeometry,
  properties: Record<string, any>,
): boolean {
  // Try to find any coordinate data
  const coords = findCoordinatesInGeometry(geometry);

  if (coords && coords.length > 0) {
    const positions = convertCoordinatesToPositions(coords);

    if (positions.length === 1) {
      // Point geometry
      entity.position = new ConstantPositionProperty(positions[0]);
      entity.point = new PointGraphics({
        pixelSize: new ConstantProperty(8),
        color: new ConstantProperty(Color.YELLOW),
        outlineColor: new ConstantProperty(Color.BLACK),
        outlineWidth: new ConstantProperty(2),
      });
    } else if (positions.length >= 2) {
      // Line or polygon geometry
      if (positions.length >= 3) {
        entity.polygon = new PolygonGraphics({
          hierarchy: new ConstantProperty(new PolygonHierarchy(positions)),
          material: createGenericMaterial(),
          outline: new ConstantProperty(true),
          outlineColor: new ConstantProperty(Color.WHITE),
        });
      } else {
        entity.polyline = new PolylineGraphics({
          positions: new ConstantProperty(positions),
          width: new ConstantProperty(2),
          material: new ColorMaterialProperty(Color.CYAN),
        });
      }
    }

    const propertyBag = new PropertyBag(properties);
    propertyBag.addProperty("cityGmlType", "Generic");
    entity.properties = propertyBag;

    return true;
  }

  return false;
}

/**
 * Find coordinates in nested CityGML geometry structure
 */
function findCoordinatesInGeometry(geometry: any): number[][] | null {
  if (!geometry || typeof geometry !== "object") {
    return null;
  }

  // Direct coordinates property
  if (geometry.coordinates && Array.isArray(geometry.coordinates)) {
    return geometry.coordinates;
  }

  // Search nested properties
  for (const [key, value] of Object.entries(geometry)) {
    if (key === "coordinates" && Array.isArray(value)) {
      return value as number[][];
    }

    // Recursively search nested objects
    if (typeof value === "object" && value !== null) {
      const found = findCoordinatesInGeometry(value);
      if (found) {
        return found;
      }
    }
  }

  return null;
}

/**
 * Convert coordinate arrays to Cesium Cartesian3 positions
 * Handles both array format [x,y,z] and object format {x,y,z}
 */
function convertCoordinatesToPositions(coordinates: any[]): Cartesian3[] {
  return coordinates
    .filter((coord) => coord != null)
    .map((coord) => {
      // Handle object format {x, y, z}
      if (
        typeof coord === "object" &&
        coord.x !== undefined &&
        coord.y !== undefined
      ) {
        return Cartesian3.fromDegrees(coord.x, coord.y, coord.z || 0);
      }
      // Handle array format [x, y, z]
      if (Array.isArray(coord) && coord.length >= 2) {
        return Cartesian3.fromDegrees(coord[0], coord[1], coord[2] || 0);
      }
      return null;
    })
    .filter((position): position is Cartesian3 => position !== null);
}

/**
 * Extract building name from geometry
 */
function extractBuildingName(geometry: CityGmlGeometry): string | null {
  const nameSources = [
    geometry.name,
    geometry.building?.name,
    geometry.gml_id,
    geometry.id,
  ];

  for (const name of nameSources) {
    if (typeof name === "string" && name.trim().length > 0) {
      return name.trim();
    }
  }

  return null;
}

/**
 * Create material for surface geometry with texture support
 */
function createSurfaceMaterial(
  geometry: CityGmlGeometry,
  properties: Record<string, any>,
):
  | ColorMaterialProperty
  | ImageMaterialProperty
  | CheckerboardMaterialProperty {
  // Check for surface-specific texture information
  const materials =
    geometry.materials || geometry.material || properties?.materials;
  const textures =
    geometry.textures || geometry.texture || properties?.textures;
  const surfaceType = geometry.surfaceType || properties?.type;

  // Handle texture materials first
  if (textures?.url || materials?.textureUrl) {
    const textureUrl = textures.url || materials.textureUrl;
    if (typeof textureUrl === "string" && textureUrl.trim().length > 0) {
      return new ImageMaterialProperty({
        image: textureUrl,
        transparent: true,
      });
    }
  }

  // Surface type-specific materials with patterns
  if (surfaceType && typeof surfaceType === "string") {
    const material = getSurfaceMaterialByType(surfaceType.toLowerCase());
    if (material) {
      return material;
    }
  }

  // Default surface material
  return new ColorMaterialProperty(Color.LIGHTGRAY.withAlpha(0.8));
}

/**
 * Get surface material based on type
 */
function getSurfaceMaterialByType(
  surfaceType: string,
): ColorMaterialProperty | CheckerboardMaterialProperty | null {
  switch (surfaceType) {
    case "roof":
    case "rooftop":
      return new CheckerboardMaterialProperty({
        evenColor: Color.DARKRED.withAlpha(0.8),
        oddColor: Color.RED.withAlpha(0.8),
        repeat: new Cartesian2(2, 2),
      });

    case "wall":
    case "wallsurface":
      return new ColorMaterialProperty(Color.BEIGE.withAlpha(0.8));

    case "floor":
    case "floorsurface":
      return new CheckerboardMaterialProperty({
        evenColor: Color.BROWN.withAlpha(0.8),
        oddColor: Color.SADDLEBROWN.withAlpha(0.8),
        repeat: new Cartesian2(1, 1),
      });

    case "ground":
    case "groundsurface":
      return new ColorMaterialProperty(Color.GREEN.withAlpha(0.6));

    case "window":
      return new ColorMaterialProperty(Color.LIGHTBLUE.withAlpha(0.5));

    case "door":
      return new ColorMaterialProperty(Color.BROWN.withAlpha(0.9));

    default:
      return null;
  }
}

/**
 * Create generic material
 */
function createGenericMaterial(): ColorMaterialProperty {
  return new ColorMaterialProperty(Color.CYAN.withAlpha(0.7));
}

/**
 * Create a fallback entity for CityGML data that couldn't be processed by other methods
 */
function createFallbackEntity(
  entity: Entity,
  geometry: CityGmlGeometry,
  properties: Record<string, any>,
): boolean {
  // Try to find any location data in properties or geometry
  let position: Cartesian3 | null = null;

  // Check properties for latitude/longitude
  const lat = properties?.latitude || properties?.lat || properties?.y;
  const lon =
    properties?.longitude ||
    properties?.lng ||
    properties?.lon ||
    properties?.x;
  const height = properties?.height || properties?.z || 0;

  if (typeof lat === "number" && typeof lon === "number") {
    position = Cartesian3.fromDegrees(lon, lat, height);
  }

  // If no position found, don't create a fallback entity at origin - just skip it
  if (!position) {
    return false;
  }

  // Create a simple point entity
  entity.position = new ConstantPositionProperty(position);
  entity.point = new PointGraphics({
    pixelSize: new ConstantProperty(12),
    color: new ConstantProperty(Color.ORANGE),
    outlineColor: new ConstantProperty(Color.BLACK),
    outlineWidth: new ConstantProperty(2),
    heightReference: new ConstantProperty(HeightReference.CLAMP_TO_GROUND),
  });

  // Add all available properties
  const propertyBag = new PropertyBag(properties);
  propertyBag.addProperty("cityGmlType", "Fallback");
  propertyBag.addProperty("originalGeometry", geometry);
  entity.properties = propertyBag;

  return true;
}

/**
 * Extract highest available LOD polygon data (LOD3 > LOD2) for a feature.
 * Used for on-select LOD upgrade.
 */
export function extractLodPolygons(
  feature: CityGmlFeature,
): ProcessedPolygon[] | null {
  const { geometry } = feature;
  const gmlGeometries =
    geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;

  if (!gmlGeometries || !Array.isArray(gmlGeometries)) return null;

  // Try LOD3 first, fall back to LOD2
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
  lodGeometries.forEach((geom: any) => {
    if (geom.polygons && Array.isArray(geom.polygons)) {
      allPolygons.push(...geom.polygons);
    }
  });

  if (allPolygons.length === 0) return null;

  return processPolygons(allPolygons);
}

/**
 * Upgrade a feature to higher LOD by hiding LOD1 surface entities
 * and adding LOD2/3 surface entities to the viewer.
 */
export function updateLodFeature(
  entry: {
    feature: CityGmlFeature;
    entity: EntityWithSurfaces;
  },
  lodPolygons: ProcessedPolygon[],
  viewer: Viewer,
): void {
  const { entity } = entry;

  const typeConfig = CITYGML_3D_TYPES.find((cfg) =>
    cfg.detect(entry.feature.properties),
  );

  // Hide LOD1 surface entities
  entity.surfaces?.forEach((s) => {
    s.show = false;
  });

  // Create LOD upgrade surface entities
  const lodSurfaces: Entity[] = [];
  lodPolygons
    .filter((p) => p.positions.length >= 3)
    .forEach((p, index) => {
      const material = typeConfig?.useSurfaceTypeColors
        ? p.material
        : new ColorMaterialProperty(
            typeConfig?.color ?? Color.GRAY.withAlpha(0.8),
          );

      const surfaceEntity = new Entity({
        id: `${entity.id}_lod_${index}`,
        name: `${entity.name} - ${p.surfaceType} ${index + 1}`,
        polygon: new PolygonGraphics({
          hierarchy: new ConstantProperty(new PolygonHierarchy(p.positions)),
          material,
          outline: new ConstantProperty(true),
          outlineColor: new ConstantProperty(Color.BLACK.withAlpha(0.8)),
          outlineWidth: new ConstantProperty(2),
          heightReference: new ConstantProperty(HeightReference.NONE),
          perPositionHeight: new ConstantProperty(true),
        }),
      });

      // Copy feature properties so click handlers can find _originalId
      const featureProps = entry.feature.properties;
      if (featureProps) {
        const lodPropertyBag = new PropertyBag(featureProps);
        lodPropertyBag.addProperty("surfaceType", p.surfaceType);
        lodPropertyBag.addProperty("lodUpgrade", true);
        surfaceEntity.properties = lodPropertyBag;
      }

      viewer.entities.add(surfaceEntity);
      lodSurfaces.push(surfaceEntity);
    });

  entity.lodSurfaces = lodSurfaces;
}

/**
 * Revert a feature back to LOD1 by removing LOD upgrade entities
 * and re-showing the original LOD1 surface entities.
 */
export function revertLodFeature(
  entity: EntityWithSurfaces,
  viewer: Viewer,
): void {
  // Remove LOD upgrade entities from viewer
  entity.lodSurfaces?.forEach((s) => {
    viewer.entities.remove(s);
  });
  entity.lodSurfaces = undefined;

  // Show LOD1 surface entities
  entity.surfaces?.forEach((s) => {
    s.show = true;
  });
}
