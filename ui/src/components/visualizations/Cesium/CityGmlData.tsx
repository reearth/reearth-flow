import { Entity } from "cesium";
import { memo, useEffect, useRef } from "react";
import { useCesium } from "resium";

import {
  convertFeatureToEntity,
  updateLodFeature,
  revertLodFeature,
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

  // Process CityGML data and create entities (only on data change)
  useEffect(() => {
    if (!cityGmlData || !viewer) return;

    // Clear existing entities
    // Before clearing the feature map, remove any active LOD primitives
    featureMapRef.current.forEach(({ entity }) => {
      const entityAny = entity as any;
      if (entityAny && entityAny.lodPrimitive) {
        viewer.scene.primitives.remove(entityAny.lodPrimitive);
        entityAny.lodPrimitive = undefined;
      }
    });

    entitiesRef.current.forEach((entity) => {
      viewer.entities.remove(entity);
    });

    featureMapRef.current.clear();

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

  // Handle LOD upgrade/revert when selectedFeatureId changes
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
        revertLodFeature(prevEntry.entity, viewer);
      }
    }

    // Upgrade newly selected feature to highest available LOD
    if (currentId) {
      const entry = featureMapRef.current.get(currentId);
      if (entry) {
        const lodPolygons = extractLodPolygons(entry.feature);
        if (lodPolygons && lodPolygons.length > 0) {
          updateLodFeature(entry, lodPolygons, viewer);
        }
      }
    }
  }, [selectedFeatureId, viewer]);

  // Cleanup on unmount
  useEffect(() => {
    // Capture the current value of featureMapRef at effect creation
    const featureMapSnapshot = featureMapRef.current;
    return () => {
      if (viewer) {
        // Remove any active LOD primitives
        featureMapSnapshot.forEach(({ entity }) => {
          if (entity.lodPrimitive) {
            viewer.scene.primitives.remove(entity.lodPrimitive);
          }
        });
        entitiesRef.current.forEach((entity) => {
          viewer.entities.remove(entity);
        });
      }
    };
  }, [viewer]);

  return null; // This component doesn't render anything directly
};

export default memo(CityGmlData);
