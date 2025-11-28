import { Entity } from "cesium";
import { memo, useCallback, useEffect, useRef } from "react";
import { useCesium } from "resium";

import { convertFeatureToEntity } from "./utils/cityGmlGeometryConverter";

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
};

const CityGmlData: React.FC<Props> = ({ cityGmlData }) => {
  const { viewer } = useCesium();
  const entitiesRef = useRef<Entity[]>([]);

  const createCityGmlEntity = useCallback(
    (feature: CityGmlFeature): Entity | null => {
      return convertFeatureToEntity(feature);
    },
    [],
  );

  // Process CityGML data and create entities
  useEffect(() => {
    if (!cityGmlData || !viewer) return;

    // Clear existing entities
    entitiesRef.current.forEach((entity) => {
      viewer.entities.remove(entity);
    });

    const newEntities: Entity[] = [];

    cityGmlData.features.forEach((feature) => {
      const entity = createCityGmlEntity(feature);
      if (entity) {
        viewer.entities.add(entity);
        newEntities.push(entity);

        // Add surface entities if they exist (for buildings with walls/roofs/floors)
        const entityWithSurfaces = entity as any;
        if (
          entityWithSurfaces.surfaces &&
          Array.isArray(entityWithSurfaces.surfaces)
        ) {
          entityWithSurfaces.surfaces.forEach((surfaceEntity: any) => {
            viewer.entities.add(surfaceEntity);
            newEntities.push(surfaceEntity);
          });
        }

        // Add additional polygon entities if they exist (for zones with multiple polygons)
        if (
          entityWithSurfaces.additionalPolygons &&
          Array.isArray(entityWithSurfaces.additionalPolygons)
        ) {
          entityWithSurfaces.additionalPolygons.forEach(
            (additionalEntity: any) => {
              viewer.entities.add(additionalEntity);
              newEntities.push(additionalEntity);
            },
          );
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
  }, [cityGmlData, viewer, createCityGmlEntity]);

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
