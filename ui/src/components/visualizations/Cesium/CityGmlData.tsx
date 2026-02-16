import { Entity } from "cesium";
import { memo, useEffect, useRef } from "react";
import { useCesium } from "resium";

import {
  convertFeatureToEntity,
  extractLodPolygons,
  updateLodFeature,
  revertLodFeature,
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
  const featureMapRef = useRef<
    Map<string, { feature: CityGmlFeature; entity: EntityWithSurfaces }>
  >(new Map());
  const prevSelectedRef = useRef<string | null>(null);

  // Process CityGML data and create entities (only on data change)
  useEffect(() => {
    if (!cityGmlData || !viewer) return;

    // Revert any active LOD upgrades before clearing
    featureMapRef.current.forEach(({ entity }) => {
      revertLodFeature(entity, viewer);
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

        // Add additional polygon entities (non-3D features like zones, land use)
        const entityAny = entity as any;
        if (entityAny.additionalPolygons) {
          entityAny.additionalPolygons.forEach((additionalEntity: Entity) => {
            viewer.entities.add(additionalEntity);
            newEntities.push(additionalEntity);
          });
        }

        // Track the primary entity by feature ID for LOD upgrades
        const fid = feature.properties._originalId;
        if (fid) {
          featureMapRef.current.set(fid, { feature, entity });
        }
      }
    });

    entitiesRef.current = newEntities;

    // Zoom to entities if any were created
    if (newEntities.length > 0) {
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

    // Revert previously selected feature back to LOD1
    if (prevId && prevId !== currentId) {
      const prevEntry = featureMapRef.current.get(prevId);
      if (prevEntry) {
        revertLodFeature(prevEntry.entity, viewer);
      }
    }

    // Upgrade newly selected feature to highest available LOD
    if (currentId) {
      const entry = featureMapRef.current.get(currentId);
      if (entry && !entry.entity.lodSurfaces) {
        const lodPolygons = extractLodPolygons(entry.feature);
        if (lodPolygons && lodPolygons.length > 0) {
          updateLodFeature(entry, lodPolygons, viewer);
        }
      }
    }
  }, [selectedFeatureId, viewer]);

  // Cleanup on unmount
  useEffect(() => {
    const featureMapSnapshot = featureMapRef.current;
    return () => {
      if (viewer) {
        featureMapSnapshot.forEach(({ entity }) => {
          revertLodFeature(entity, viewer);
        });
        entitiesRef.current.forEach((entity) => {
          viewer.entities.remove(entity);
        });
      }
    };
  }, [viewer]);

  return null;
};

export default memo(CityGmlData);
