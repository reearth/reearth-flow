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
} from "cesium";

// Extend Entity type to include surfaces
type EntityWithSurfaces = Entity & {
  surfaces?: Entity[];
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
export function convertFeatureToEntity(feature: CityGmlFeature): EntityWithSurfaces | null {
  const { geometry, properties, id } = feature;

  if (!geometry || geometry.type !== "CityGmlGeometry") {
    return null;
  }

  const entity = new Entity({
    id: id || `citygml-${Math.random().toString(36).substr(2, 9)}`,
    name: properties?.name || extractBuildingName(geometry) || "CityGML Feature",
  });

  // Try different geometry conversion strategies
  if (convertBuildingGeometry(entity, geometry, properties)) {
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
 * Convert building-specific geometry (solid, multi-surface, etc.)
 */
function convertBuildingGeometry(
  entity: EntityWithSurfaces,
  geometry: CityGmlGeometry,
  properties: Record<string, any>
): boolean {
  // Access the CityGML geometry structure - try both possible locations
  const gmlGeometries = geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
  
  const hasBuildingAttributes = properties && (
    properties["bldg:measuredHeight"] || 
    properties["bldg:usage"] || 
    properties["bldg:class"] ||
    properties.gmlName?.includes("bldg:") ||
    properties.cityGmlAttributes?.["bldg:measuredHeight"] ||
    properties.cityGmlAttributes?.["bldg:usage"] ||
    properties.cityGmlAttributes?.["bldg:class"]
  );

  if (!gmlGeometries || !Array.isArray(gmlGeometries) || !hasBuildingAttributes) {
    return false;
  }

  // Group geometries by LOD and create meaningful building entities
  const lod1Geometries = gmlGeometries.filter(geom => geom.lod === 1);
  
  
  let hasValidGeometry = false;
  entity.surfaces = [];

  // Skip LOD0 footprints since we have proper 3D geometry
  // (Footprints would be hidden under the brown floor polygons anyway)

  // Create LOD1+ 3D building geometry (walls + roof as single entity)
  if (lod1Geometries.length > 0) {
    const buildingPolygons: any[] = [];
    lod1Geometries.forEach(geom => {
      if (geom.polygons && Array.isArray(geom.polygons)) {
        buildingPolygons.push(...geom.polygons);
      }
    });

    if (buildingPolygons.length > 0) {
      // Show ALL LOD1 polygons and categorize them properly
      const limitedPolygons = buildingPolygons; // Show all, not just 3
      
      limitedPolygons.forEach((polygon, index) => {
        if (polygon.exterior && Array.isArray(polygon.exterior)) {
          // Detect surface type by Z-coordinate pattern
          const zValues = polygon.exterior.map((coord: any) => coord.z || 0);
          const minZ = Math.min(...zValues);
          const maxZ = Math.max(...zValues);
          const isFlat = Math.abs(maxZ - minZ) < 0.1; // All points at same height
          
          let surfaceType: string;
          let material: ColorMaterialProperty;
          
          if (isFlat && minZ < (Math.min(...buildingPolygons.flatMap((p: any) => p.exterior.map((c: any) => c.z))) + 1)) {
            // Flat surface at low elevation = Floor
            surfaceType = "Floor";
            material = new ColorMaterialProperty(Color.BROWN.withAlpha(0.8));
          } else if (isFlat) {
            // Flat surface at high elevation = Roof  
            surfaceType = "Roof";
            material = new ColorMaterialProperty(Color.RED.withAlpha(0.8));
          } else {
            // Varying heights = Wall
            surfaceType = "Wall";
            material = new ColorMaterialProperty(Color.BLUE.withAlpha(0.8));
          }
          
          // Get ground level for normalization
          const groundLevel = Math.min(...buildingPolygons.flatMap((p: any) => p.exterior.map((c: any) => c.z || 0)));
          
          // Convert coordinates and make them relative to ground level
          const rawPositions = convertCoordinatesToPositions(polygon.exterior);
          const positions = rawPositions.map(pos => {
            const cartographic = Cartographic.fromCartesian(pos);
            // Offset height to be relative to ground (subtract minimum Z)
            cartographic.height = (cartographic.height || 0) - groundLevel;
            return Cartographic.toCartesian(cartographic);
          });
          
          if (positions.length >= 3) {
            const surfaceEntity = new Entity({
              id: `${entity.id}_${surfaceType.toLowerCase()}_${index}`,
              name: `${entity.name} - ${surfaceType} ${index + 1}`,
              polygon: new PolygonGraphics({
                hierarchy: new ConstantProperty(new PolygonHierarchy(positions)),
                material: material,
                outline: new ConstantProperty(true),
                outlineColor: new ConstantProperty(Color.BLACK.withAlpha(0.8)),
                outlineWidth: new ConstantProperty(2),
                heightReference: new ConstantProperty(HeightReference.NONE),
                perPositionHeight: new ConstantProperty(true), // Enable per-vertex height
              }),
            });

            const propertyBag = new PropertyBag(properties);
            propertyBag.addProperty("cityGmlType", surfaceType);
            propertyBag.addProperty("lod", 1);
            propertyBag.addProperty("surfaceIndex", index + 1);
            propertyBag.addProperty("totalLOD1Polygons", buildingPolygons.length);
            propertyBag.addProperty("zRange", `${minZ.toFixed(2)}-${maxZ.toFixed(2)}m`);
            surfaceEntity.properties = propertyBag;

            if (!entity.surfaces) entity.surfaces = [];
            entity.surfaces.push(surfaceEntity);
            hasValidGeometry = true;
          }
        }
      });
    }
  }

  if (hasValidGeometry) {
    // Store CityGML-specific properties on the main entity
    const propertyBag = new PropertyBag(properties);
    propertyBag.addProperty("cityGmlType", "Building");
    propertyBag.addProperty("height", extractHeight(geometry, properties));
    propertyBag.addProperty("surfaceCount", entity.surfaces ? entity.surfaces.length : 0);
    entity.properties = propertyBag;

    return true;
  }

  return false;
}

/**
 * Convert surface geometry (walls, roofs, floors, etc.)
 */
function convertSurfaceGeometry(
  entity: Entity,
  geometry: CityGmlGeometry,
  properties: Record<string, any>
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
      propertyBag.addProperty("surfaceType", geometry.surfaceType || properties?.type);
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
  properties: Record<string, any>
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
 * Extract building height from various CityGML properties
 */
function extractHeight(
  geometry: CityGmlGeometry,
  properties: Record<string, any>
): number {
  // Try different height properties from CityGML structure
  const heightSources = [
    properties?.["bldg:measuredHeight"],
    properties?.["uro:BuildingDetailAttribute_uro:buildingHeight"],
    properties?.cityGmlAttributes?.["bldg:measuredHeight"],
    properties?.cityGmlAttributes?.uro?.BuildingDetailAttribute?.uro?.buildingHeight,
    properties?.height,
    properties?.building_height,
    properties?.HEIGHT,
  ];

  for (const height of heightSources) {
    if (typeof height === "number" && height > 0) {
      return height;
    }
    if (typeof height === "string") {
      const parsed = parseFloat(height);
      if (!isNaN(parsed) && parsed > 0) {
        return parsed;
      }
    }
  }

  // If no height found in properties, try to calculate from geometry
  const gmlGeometries = geometry.gmlGeometries || geometry.value?.cityGmlGeometry?.gmlGeometries;
  
  if (gmlGeometries && Array.isArray(gmlGeometries)) {
    let minZ = Infinity;
    let maxZ = -Infinity;
    
    // Find min and max Z values from all polygons
    gmlGeometries.forEach(geom => {
      if (geom.polygons && Array.isArray(geom.polygons)) {
        geom.polygons.forEach((polygon: any) => {
          if (polygon.exterior && Array.isArray(polygon.exterior)) {
            polygon.exterior.forEach((coord: any) => {
              if (typeof coord === 'object' && typeof coord.z === 'number') {
                minZ = Math.min(minZ, coord.z);
                maxZ = Math.max(maxZ, coord.z);
              }
            });
          }
        });
      }
    });
    
    if (minZ !== Infinity && maxZ !== -Infinity) {
      const calculatedHeight = maxZ - minZ;
      if (calculatedHeight > 0) {
        return calculatedHeight;
      }
    }
  }

  return 10; // Default height
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
      if (typeof coord === 'object' && coord.x !== undefined && coord.y !== undefined) {
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
  properties: Record<string, any>
): ColorMaterialProperty | ImageMaterialProperty | CheckerboardMaterialProperty {
  // Check for surface-specific texture information
  const materials = geometry.materials || geometry.material || properties?.materials;
  const textures = geometry.textures || geometry.texture || properties?.textures;
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
  surfaceType: string
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
  properties: Record<string, any>
): boolean {
  // Try to find any location data in properties or geometry
  let position: Cartesian3 | null = null;

  // Check properties for latitude/longitude
  const lat = properties?.latitude || properties?.lat || properties?.y;
  const lon = properties?.longitude || properties?.lng || properties?.lon || properties?.x;
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
