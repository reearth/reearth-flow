import { GeoJsonDataSource } from "cesium";
import { memo, useEffect, useRef } from "react";
import { useCesium } from "resium";

type Props = {
  geoJsonData: any | null;
  entityMapRef?: React.RefObject<Map<string | number, any>>;
};

const GeoJsonData: React.FC<Props> = ({ geoJsonData, entityMapRef }) => {
  const { viewer } = useCesium();
  const dataSourceRef = useRef<GeoJsonDataSource | null>(null);
  const isInitialLoadRef = useRef(true);

  useEffect(() => {
    if (!viewer || !geoJsonData || !entityMapRef) return;

    // Get existing feature IDs for comparison
    const existingIds = new Set(entityMapRef.current.keys());
    const newIds = new Set<string | number>();
    const newFeatureMap = new Map<string | number, any>();

    // Build map of new features by ID
    geoJsonData.features?.forEach((feature: any) => {
      if (feature.id !== undefined) {
        newIds.add(feature.id);
        newFeatureMap.set(feature.id, feature);
      }
    });

    // Determine what changed
    const idsToAdd = Array.from(newIds).filter((id) => !existingIds.has(id));
    const idsToRemove = Array.from(existingIds).filter((id) => !newIds.has(id));

    // If this is the initial load or we need a full refresh (too many changes)
    const needsFullRefresh =
      isInitialLoadRef.current ||
      !dataSourceRef.current ||
      idsToAdd.length + idsToRemove.length > newIds.size * 0.5;

    if (needsFullRefresh) {
      // Full load: Remove existing data source and create new one
      if (
        dataSourceRef.current &&
        viewer.dataSources.contains(dataSourceRef.current)
      ) {
        viewer.dataSources.remove(dataSourceRef.current);
      }

      // Load new data source
      GeoJsonDataSource.load(geoJsonData)
        .then((newDataSource) => {
          dataSourceRef.current = newDataSource;
          viewer.dataSources.add(newDataSource);

          // Build entity ID map with O(1) lookup
          const newEntityMap = new Map<string | number, any>();

          newDataSource.entities.values.forEach((entity) => {
            let featureId: string | number | undefined;

            // Extract feature ID from entity
            if (entity.properties) {
              const props = entity.properties.getValue();
              if (props) {
                // Try to find the original feature ID
                const matchingFeature = geoJsonData.features?.find(
                  (feature: any) => {
                    if (feature.id !== undefined && props.id === feature.id) {
                      return true;
                    }
                    // Fallback: compare properties
                    if (feature.properties) {
                      const propKeys = Object.keys(feature.properties);
                      return propKeys.every(
                        (key) => props[key] === feature.properties[key],
                      );
                    }
                    return false;
                  },
                );

                if (matchingFeature?.id !== undefined) {
                  featureId = matchingFeature.id;
                  // Store original ID in entity properties for future reference
                  entity.properties.addProperty("_originalId", featureId);
                }
              }
            }

            // Store entity in map
            if (featureId !== undefined) {
              newEntityMap.set(featureId, entity);
            }
          });

          entityMapRef.current = newEntityMap;

          // Zoom to entities on initial load
          if (isInitialLoadRef.current) {
            viewer.zoomTo(newDataSource.entities);
            isInitialLoadRef.current = false;
          }

          // TODO: Add more styling options
          // if (entity.polygon) {
          //   entity.polygon.material = new ColorMaterialProperty(
          //     Color.BLACK.withAlpha(0.5),
          //   );
          //   entity.polygon.outlineColor = new ColorMaterialProperty(
          //     Color.BLACK,
          //   );
          // }
        })
        .catch((error) => {
          console.error("Error loading GeoJSON data:", error);
        });
    } else {
      // Incremental update: Only add/remove changed entities
      const currentDataSource = dataSourceRef.current;
      if (!currentDataSource) return;

      // Remove entities that are no longer in the data
      idsToRemove.forEach((id) => {
        const entity = entityMapRef.current.get(id);
        if (entity) {
          currentDataSource.entities.remove(entity);
          entityMapRef.current.delete(id);
        }
      });

      // Add new entities
      if (idsToAdd.length > 0) {
        const newFeatures = idsToAdd
          .map((id) => newFeatureMap.get(id))
          .filter(Boolean);

        const newGeoJson = {
          type: "FeatureCollection",
          features: newFeatures,
        };

        GeoJsonDataSource.load(newGeoJson)
          .then((tempDataSource) => {
            // Transfer entities from temp data source to main data source
            tempDataSource.entities.values.forEach((entity) => {
              const props = entity.properties?.getValue();
              let featureId: string | number | undefined;

              // Find the feature ID for this entity
              if (props) {
                const matchingFeature = newFeatures.find((feature: any) => {
                  if (feature.id !== undefined && props.id === feature.id) {
                    return true;
                  }
                  if (feature.properties) {
                    const propKeys = Object.keys(feature.properties);
                    return propKeys.every(
                      (key) => props[key] === feature.properties[key],
                    );
                  }
                  return false;
                });

                if (matchingFeature?.id !== undefined) {
                  featureId = matchingFeature.id;
                  entity.properties?.addProperty("_originalId", featureId);
                }
              }

              // Add entity to main data source
              const addedEntity = currentDataSource.entities.add(entity);

              // Update entity map
              if (featureId !== undefined) {
                entityMapRef.current.set(featureId, addedEntity);
              }
            });
          })
          .catch((error) => {
            console.error("Error loading incremental GeoJSON data:", error);
          });
      }
    }

    // Cleanup function
    return () => {
      // Don't remove data source on every render, only on unmount
    };
  }, [geoJsonData, viewer, entityMapRef]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (
        dataSourceRef.current &&
        viewer?.dataSources.contains(dataSourceRef.current)
      ) {
        viewer.dataSources.remove(dataSourceRef.current);
      }
      entityMapRef?.current.clear();
    };
  }, [viewer, entityMapRef]);

  // This component manages entities imperatively, so it doesn't render anything
  return null;
};

export default memo(GeoJsonData);
