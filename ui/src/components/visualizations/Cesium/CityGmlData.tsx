import {
  Entity,
  ConstantProperty,
  PolygonHierarchy,
  PolygonGraphics,
  Color,
  HeightReference,
} from "cesium";
import { memo, useEffect, useRef, useState } from "react";
import { useCesium } from "resium";

import {
  convertFeatureToEntity,
  extractLodPolygons,
  type EntityWithSurfaces,
} from "./utils/cityGmlGeometryConverter";

type CityGmlFeature = {
  id?: string;
  type: "Feature";
  properties: Record<string, any>;
  geometry: {
    type: "CityGmlGeometry";
    [key: string]: any;
  };
};

type Props = {
  cityGmlData: {
    type: "FeatureCollection";
    features: CityGmlFeature[];
  } | null;
  selectedFeatureId?: string | null;
};

const CityGmlData: React.FC<Props> = ({ cityGmlData, selectedFeatureId }) => {
  const { viewer } = useCesium();
  const entitiesRef = useRef<Entity[]>([]);
  // Map feature IDs to their source feature data + primary entity for LOD swaps
  const featureMapRef = useRef<
    Map<string, { feature: CityGmlFeature; entity: EntityWithSurfaces }>
  >(new Map());
  const prevSelectedRef = useRef<string | null>(null);
  // Track the count of original LOD1 surfaces per feature so we know which extras to hide on revert
  const lod1SurfaceCountRef = useRef<Map<string, number>>(new Map());
  const [dataSourceKey, setDataSourceKey] = useState(0);

  useEffect(() => {
    setDataSourceKey(dataSourceKey + 1);
  }, [cityGmlData]); // eslint-disable-line react-hooks/exhaustive-deps

  // Process CityGML data and create entities (only on data change)
  useEffect(() => {
    if (!cityGmlData || !viewer) return;

    // Clear existing entities
    entitiesRef.current.forEach((entity) => {
      viewer.entities.remove(entity);
    });

    featureMapRef.current.clear();
    lod1SurfaceCountRef.current.clear();

    const newEntities: Entity[] = [];

    cityGmlData.features.forEach((feature) => {
      const entity = convertFeatureToEntity(feature);
      if (entity) {
        // Add parent entity
        viewer.entities.add(entity);
        newEntities.push(entity);

        // Add child surface entities
        if (entity.surfaces) {
          entity.surfaces.forEach((surfaceEntity) => {
            viewer.entities.add(surfaceEntity);
            newEntities.push(surfaceEntity);
          });
        }

        // Add additional polygon entities (non-building features)
        const entityAny = entity as any;
        if (entityAny.additionalPolygons) {
          entityAny.additionalPolygons.forEach((additionalEntity: Entity) => {
            viewer.entities.add(additionalEntity);
            newEntities.push(additionalEntity);
          });
        }

        // Track the primary entity by feature ID for later LOD mutation
        const fid = feature.properties._originalId;
        if (fid) {
          featureMapRef.current.set(fid, { feature, entity });
          lod1SurfaceCountRef.current.set(fid, entity.surfaces?.length ?? 0);
        }
      }
    });

    entitiesRef.current = newEntities;

    // Zoom to entities if any were created
    if (newEntities.length > 0) {
      // Wait for the next render frame to ensure entities are rendered
      const removeListener = viewer.scene.postRender.addEventListener(() => {
        removeListener();
        viewer.zoomTo(viewer.entities);
      });
    }
  }, [cityGmlData, viewer]);

  // Handle LOD2 in-place mutation when selectedFeatureId changes
  useEffect(() => {
    if (!viewer) return;

    const prevId = prevSelectedRef.current;
    const currentId = selectedFeatureId ?? null;
    prevSelectedRef.current = currentId;

    if (prevId === currentId) return;

    // Revert previously selected feature back to LOD1
    if (prevId) {
      const prevEntry = featureMapRef.current.get(prevId);
      if (prevEntry) {
        const { entity } = prevEntry;
        const lodData = entity.lodData;
        const surfaces = entity.surfaces;
        const lod1Count = lod1SurfaceCountRef.current.get(prevId) ?? 0;

        if (surfaces && lodData) {
          for (let i = 0; i < surfaces.length; i++) {
            if (i < lod1Count && i < lodData.length) {
              // Revert to LOD1 positions in-place
              const hierarchy = surfaces[i].polygon
                ?.hierarchy as ConstantProperty;
              hierarchy?.setValue(new PolygonHierarchy(lodData[i]));
              surfaces[i].show = true;
            } else {
              // Extra surface added for LOD2 - hide it
              surfaces[i].show = false;
            }
          }
        }
      }
    }

    // Upgrade newly selected feature to LOD2
    if (currentId) {
      const entry = featureMapRef.current.get(currentId);
      if (entry) {
        const lod2Polygons = extractLodPolygons(entry.feature);
        if (lod2Polygons) {
          const { entity } = entry;
          const surfaces = entity.surfaces ?? [];

          // Mutate existing surfaces or add new ones for LOD2
          for (let i = 0; i < lod2Polygons.length; i++) {
            if (i < surfaces.length) {
              // Mutate existing surface entity in-place via ConstantProperty.setValue()
              const hierarchy = surfaces[i].polygon
                ?.hierarchy as ConstantProperty;
              hierarchy?.setValue(
                new PolygonHierarchy(lod2Polygons[i].positions),
              );
              surfaces[i].show = true;
            } else {
              // LOD2 has more polygons than LOD1 - create extra surface entities
              const surfaceEntity = new Entity({
                id: `${entity.id}_lod2_extra_${i}`,
                name: `${entity.name} - ${lod2Polygons[i].surfaceType} ${i + 1}`,
                polygon: new PolygonGraphics({
                  hierarchy: new ConstantProperty(
                    new PolygonHierarchy(lod2Polygons[i].positions),
                  ),
                  material: lod2Polygons[i].material,
                  outline: new ConstantProperty(true),
                  outlineColor: new ConstantProperty(
                    Color.BLACK.withAlpha(0.8),
                  ),
                  outlineWidth: new ConstantProperty(2),
                  heightReference: new ConstantProperty(HeightReference.NONE),
                  perPositionHeight: new ConstantProperty(true),
                }),
              });
              viewer.entities.add(surfaceEntity);
              surfaces.push(surfaceEntity);
              entitiesRef.current.push(surfaceEntity);
            }
          }

          // Update surfaces reference in case extras were added
          entity.surfaces = surfaces;
        }
      }
    }
  }, [selectedFeatureId, viewer]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (viewer && entitiesRef.current.length > 0) {
        entitiesRef.current.forEach((entity) => {
          viewer.entities.remove(entity);
        });
      }
    };
  }, [viewer]);

  return null; // This component doesn't render anything directly
};

export default memo(CityGmlData);
