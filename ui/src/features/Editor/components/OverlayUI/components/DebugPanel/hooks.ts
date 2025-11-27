import bbox from "@turf/bbox";
import { Cartesian3, GeoJsonDataSource } from "cesium";
import {
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { useStreamingDebugRunQuery } from "@flow/hooks/useStreamingDebugRunQuery";
import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const prevIntermediateDataUrls = useRef<string[] | undefined>(undefined);
  const [fullscreenDebug, setFullscreenDebug] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const previousSelectedFeature = useRef<any>(null);

  // const [enableClustering, setEnableClustering] = useState<boolean>(true);
  const [selectedFeature, setSelectedFeature] = useState<any>(null);
  const handlePreviousSelectedFeature = useCallback((feature: any) => {
    previousSelectedFeature.current = feature;
  }, []);

  const handleSelectedFeature = useCallback(
    (feature: any) => {
      const currId = selectedFeature?.id;
      const prevId = previousSelectedFeature.current?.id;
      if (currId !== feature?.id) {
        setSelectedFeature(feature);
      }
      if (currId !== prevId) {
        handlePreviousSelectedFeature(selectedFeature);
      }
    },
    [selectedFeature, handlePreviousSelectedFeature],
  );

  const [convertedSelectedFeature, setConvertedSelectedFeature] =
    useState(null);
  const mapRef = useRef<maplibregl.Map | null>(null);
  const cesiumViewerRef = useRef<any>(null);

  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );
  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const { useGetJob } = useJob();

  const { job: debugJob } = useGetJob(debugJobState?.jobId ?? "");

  const outputURLs = useMemo(() => debugJob?.outputURLs, [debugJob]);

  const intermediateDataURLs = useMemo(
    () => debugJobState?.selectedIntermediateData?.map((sid) => sid.url),
    [debugJobState],
  );

  // Separate intermediate data URLs (for dropdown) from output data URLs (for download)
  const dataURLs = useMemo(() => {
    const urls: { key: string; name: string }[] = [];
    if (debugJobState?.selectedIntermediateData) {
      debugJobState.selectedIntermediateData.forEach((selectedData) => {
        urls.push({
          key: selectedData.url,
          name:
            selectedData.displayName ||
            selectedData.url.split("/").pop() ||
            selectedData.url,
        });
      });
    }
    // Remove output data from dropdown - now handled separately
    return urls.length ? urls : undefined;
  }, [debugJobState?.selectedIntermediateData]);

  // Separate output data for download functionality
  const outputDataForDownload = useMemo(() => {
    if (!outputURLs) return undefined;
    return outputURLs.map((url) => ({
      url,
      name: url.split("/").pop() || url,
    }));
  }, [outputURLs]);

  const [selectedDataURL, setSelectedDataURL] = useState<string | undefined>(
    undefined,
  );

  useEffect(() => {
    if (intermediateDataURLs !== prevIntermediateDataUrls.current) {
      const newURL = intermediateDataURLs?.find(
        (url) => !prevIntermediateDataUrls.current?.includes(url),
      );
      setSelectedDataURL(newURL);
      prevIntermediateDataUrls.current = intermediateDataURLs;
      setMinimized(false);
    } else if (
      (dataURLs?.length && !selectedDataURL) ||
      (selectedDataURL && !dataURLs?.find((u) => u.key === selectedDataURL))
    ) {
      setSelectedDataURL(dataURLs?.[0].key);
    }
  }, [dataURLs, selectedDataURL, intermediateDataURLs]);

  const handleSelectedDataChange = (url: string) => {
    setSelectedDataURL(url);
    setMinimized(false);
  };

  // First, get metadata to determine file size
  const metadataUrl =
    selectedDataURL ?? (dataURLs?.length ? dataURLs[0].key : "");

  const streamingQuery = useStreamingDebugRunQuery(metadataUrl, {
    enabled: !!metadataUrl,
  });

  const selectedOutputData = streamingQuery.fileContent;
  const fileType = streamingQuery.fileType;

  const handleExpand = () => {
    setExpanded((prev) => !prev);
  };

  const handleMinimize = (e: MouseEvent) => {
    e.stopPropagation();
    setMinimized((prev) => !prev);
  };

  const handleTabChange = () => {
    if (minimized) {
      setMinimized(false);
    }
  };

  const handleFullscreenExpand = () => {
    setFullscreenDebug((prev) => !prev);
  };

  const handleFlyToSelectedFeature = useCallback(
    (selectedFeature: any) => {
      if (!selectedFeature) return;

      // Get the current geometry type
      const currentDetectedGeometryType = streamingQuery.detectedGeometryType;

      // Determine which viewer to use based on detected geometry type
      const is3D =
        currentDetectedGeometryType === "CityGmlGeometry" ||
        currentDetectedGeometryType === "FlowGeometry3D";

      if (is3D && cesiumViewerRef.current) {
        // 3D Cesium viewer - zoom to entities by feature ID
        try {
          // Access the actual Cesium viewer from Resium component
          const cesiumViewer = cesiumViewerRef.current?.cesiumElement;

          if (!cesiumViewer) {
            console.warn("Cesium viewer not initialized yet");
            return;
          }

          const featureId = selectedFeature.id;
          if (!featureId) {
            console.warn("No feature ID found for Cesium zoom");
            return;
          }

          // Safety check for entities collection
          if (!cesiumViewer.entities || !cesiumViewer.entities.values) {
            console.warn("Cesium entities collection not available yet");
            return;
          }

          // Find all entities that belong to this feature
          const matchingEntities = cesiumViewer.entities.values.filter(
            (entity: any) => {
              // Method 1: Direct entity ID match (CityGML entities)
              if (
                entity.id === featureId ||
                JSON.stringify(entity.id) === JSON.stringify(featureId)
              ) {
                return true;
              }

              // Method 2: Check buildingId property (CityGML surface entities)
              const buildingId = entity.properties?.getValue()?.buildingId;
              if (
                buildingId &&
                (buildingId === featureId ||
                  JSON.stringify(buildingId) === JSON.stringify(featureId))
              ) {
                return true;
              }

              // Method 3: Check compound ID prefix (CityGML surface entities like "buildingId_wall_1")
              if (
                entity.id &&
                typeof entity.id === "string" &&
                entity.id.includes("_")
              ) {
                const baseId = entity.id.split("_")[0];
                if (
                  baseId === featureId ||
                  JSON.stringify(baseId) === JSON.stringify(featureId)
                ) {
                  return true;
                }
              }

              // Method 4: Check GeoJSON entity properties for original feature data
              try {
                const entityProps = entity.properties?.getValue();
                if (entityProps) {
                  // Check if this entity has original GeoJSON feature data
                  const originalId = entityProps.id;
                  if (
                    originalId === featureId ||
                    JSON.stringify(originalId) === JSON.stringify(featureId)
                  ) {
                    return true;
                  }

                  // Also check all property keys for potential ID matches
                  for (const [key, value] of Object.entries(entityProps)) {
                    if (
                      key.toLowerCase().includes("id") &&
                      (value === featureId ||
                        JSON.stringify(value) === JSON.stringify(featureId))
                    ) {
                      return true;
                    }
                  }
                }
              } catch {
                // Silent fail for property access errors
              }

              return false;
            },
          );

          if (matchingEntities.length > 0) {
            // Validate entities have reasonable coordinates
            const validEntities = matchingEntities.filter((entity: any) => {
              try {
                if (entity.polygon?.hierarchy?.getValue) {
                  const hierarchy = entity.polygon.hierarchy.getValue();
                  if (hierarchy?.positions) {
                    // Check if any position has invalid coordinates
                    return hierarchy.positions.every(
                      (pos: any) =>
                        pos &&
                        typeof pos.x === "number" &&
                        !isNaN(pos.x) &&
                        isFinite(pos.x) &&
                        typeof pos.y === "number" &&
                        !isNaN(pos.y) &&
                        isFinite(pos.y) &&
                        typeof pos.z === "number" &&
                        !isNaN(pos.z) &&
                        isFinite(pos.z),
                    );
                  }
                }
                return true; // If no polygon, assume valid
              } catch {
                return false;
              }
            });

            if (validEntities.length === 0) {
              console.warn(
                "No valid entities found - all have invalid coordinates",
              );
              return;
            }

            try {
              // Try different zoom approaches to handle potential coordinate issues

              // Approach 1: Simple zoomTo without offset on valid entities
              cesiumViewer.zoomTo(validEntities);
            } catch (zoomError) {
              console.warn(
                "Direct zoomTo failed, trying fallback approach:",
                zoomError,
              );

              try {
                // Approach 2: Zoom to first valid entity only
                if (validEntities[0]) {
                  cesiumViewer.zoomTo(validEntities[0]);
                }
              } catch (fallbackError) {
                console.error("All zoom approaches failed:", fallbackError);

                // Approach 3: Manual camera positioning using entity bounds
                try {
                  const entity = validEntities[0];
                  if (entity.position) {
                    const position = entity.position.getValue();
                    if (position) {
                      cesiumViewer.camera.lookAt(
                        position,
                        new Cartesian3(100, 100, 100), // Simple offset
                      );
                    }
                  }
                } catch (manualError) {
                  console.error(
                    "Manual camera positioning failed:",
                    manualError,
                  );
                }
              }
            }
          } else {
            console.warn(
              "No matching Cesium entities found for feature ID:",
              featureId,
            );

            // Fallback: Try to zoom to feature using its original geometry
            // This is useful for simple GeoJSON data where entity matching fails
            try {
              if (selectedFeature.geometry) {
                console.log("Attempting fallback zoom using feature geometry");

                // Create a temporary entity from the feature geometry to zoom to
                const tempEntity = cesiumViewer.entities.add({
                  id: `temp_zoom_${featureId}`,
                  position: undefined, // Will be determined by geometry
                });

                // Try to set geometry on temp entity and zoom to it
                if (
                  selectedFeature.geometry.type === "Point" &&
                  selectedFeature.geometry.coordinates
                ) {
                  const [lng, lat, height = 0] =
                    selectedFeature.geometry.coordinates;
                  tempEntity.position = Cartesian3.fromDegrees(
                    lng,
                    lat,
                    height,
                  );

                  cesiumViewer.zoomTo(tempEntity);
                } else {
                  // For other geometry types, try to use Cesium's GeoJSON processing
                  // Create a minimal feature collection for this single feature
                  const tempGeoJSON = {
                    type: "FeatureCollection",
                    features: [selectedFeature],
                  };

                  // Load as temporary data source and zoom to it
                  GeoJsonDataSource.load(tempGeoJSON)
                    .then((dataSource: any) => {
                      cesiumViewer.dataSources.add(dataSource);
                      cesiumViewer.zoomTo(dataSource.entities);
                      // Clean up after zoom
                      setTimeout(() => {
                        cesiumViewer.dataSources.remove(dataSource);
                      }, 1000);
                    })
                    .catch(() => {
                      // Final fallback: just zoom to a reasonable area
                      console.warn(
                        "All zoom methods failed, using default view",
                      );
                    });
                }

                // Clean up temp entity
                setTimeout(() => {
                  cesiumViewer.entities.removeById(`temp_zoom_${featureId}`);
                }, 500);
              }
            } catch (fallbackError) {
              console.error("Fallback zoom method also failed:", fallbackError);
            }
          }
        } catch (err) {
          console.error("Error zooming to Cesium feature:", err);
        }
      } else if (!is3D && mapRef.current) {
        // 2D MapLibre viewer - use existing bbox approach
        try {
          const [minLng, minLat, maxLng, maxLat] = bbox(selectedFeature);
          mapRef.current.fitBounds(
            [
              [minLng, minLat],
              [maxLng, maxLat],
            ],
            { padding: 0, duration: 500, maxZoom: 15 },
          );
        } catch (err) {
          console.error("Error computing bbox for selectedFeature:", err);
        }
      }
    },
    [streamingQuery.detectedGeometryType, cesiumViewerRef, mapRef],
  );

  const handleRowSingleClick = useCallback(
    (value: any) => {
      // setEnableClustering(false);
      handleSelectedFeature(value);
    },
    [handleSelectedFeature],
  );

  const handleRowDoubleClick = useCallback(
    (value: any) => {
      // setEnableClustering(false);
      handleSelectedFeature(value);
      handleFlyToSelectedFeature(convertedSelectedFeature);
    },
    [
      convertedSelectedFeature,
      handleFlyToSelectedFeature,
      handleSelectedFeature,
    ],
  );

  return {
    debugJobId,
    debugJobState,
    fileType,
    mapRef,
    cesiumViewerRef,
    fullscreenDebug,
    expanded,
    minimized,
    selectedDataURL,
    dataURLs,
    outputDataForDownload,
    selectedOutputData,
    // enableClustering,
    selectedFeature,
    previousSelectedFeature,
    handleSelectedFeature,
    setConvertedSelectedFeature,
    // setEnableClustering,
    handleFullscreenExpand,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
    handleRowSingleClick,
    handleRowDoubleClick,
    handleFlyToSelectedFeature,

    // Data loading features (always available now)
    streamingQuery: streamingQuery,
    streamingProgress: streamingQuery.progress,
    detectedGeometryType: streamingQuery.detectedGeometryType,
    visualizerType: streamingQuery.visualizerType,
    totalFeatures: streamingQuery.totalFeatures,
    isComplete: streamingQuery.isComplete,
  };
};
