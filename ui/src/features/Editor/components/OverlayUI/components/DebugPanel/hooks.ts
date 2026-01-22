import bbox from "@turf/bbox";
import { Cartesian3, GeoJsonDataSource } from "cesium";
import { MouseEvent, useCallback, useMemo, useRef, useState } from "react";

import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useStreamingDebugRunQuery } from "@flow/hooks/useStreamingDebugRunQuery";
import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const [fullscreenDebug, setFullscreenDebug] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [detailsOverlayOpen, setDetailsOverlayOpen] = useState(false);
  const prevSelectedDataURLRef = useRef<string | undefined>(undefined);
  // const [enableClustering, setEnableClustering] = useState<boolean>(true);
  const [selectedFeatureId, setSelectedFeatureId] = useState<string | null>(
    null,
  );
  const [convertedSelectedFeature, setConvertedSelectedFeature] =
    useState(null);
  const mapRef = useRef<maplibregl.Map | null>(null);
  const cesiumViewerRef = useRef<any>(null);
  const cesiumEntityMapRef = useRef<Map<string, any>>(new Map());

  const [currentProject] = useCurrentProject();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

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

  const selectedDataURL = useMemo(() => {
    if (!debugJobState?.focusedIntermediateData) return undefined;
    return debugJobState.focusedIntermediateData;
  }, [debugJobState?.focusedIntermediateData]);

  const handleSelectedDataChange = (url: string) => {
    if (debugJobState?.focusedIntermediateData !== url) {
      updateValue({
        ...debugRunState,
        jobs:
          debugRunState?.jobs?.map((job) => {
            if (job.projectId !== currentProject?.id) return job;

            return {
              ...job,
              focusedIntermediateData: url,
            };
          }) ?? [],
      });
      setMinimized(false);
      prevSelectedDataURLRef.current = debugJobState?.focusedIntermediateData;
    }
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

          // OPTIMIZED: Use entity map for O(1) lookup instead of O(n) filtering
          const matchingEntities: any[] = [];

          // First, try direct lookup from entity map (for GeoJSON features)
          const directEntity = cesiumEntityMapRef.current.get(featureId);
          if (directEntity) {
            matchingEntities.push(directEntity);
          }

          // For CityGML features, we still need to search, but only if direct lookup failed
          // CityGML creates multiple entities (building + surfaces) with related IDs
          if (matchingEntities.length === 0) {
            // Fallback to entity search, but with optimized checks
            for (const entity of cesiumViewer.entities.values) {
              // Quick checks first (avoid expensive operations)
              if (entity.id === featureId) {
                matchingEntities.push(entity);
                continue;
              }

              // Check for compound IDs (CityGML surfaces like "buildingId_wall_1")
              if (
                typeof entity.id === "string" &&
                entity.id.startsWith(featureId + "_")
              ) {
                matchingEntities.push(entity);
                continue;
              }

              // Check buildingId property (for CityGML surface entities)
              // Only access properties if entity has them (avoid unnecessary calls)
              if (entity.properties) {
                try {
                  const props = entity.properties.getValue();
                  if (props?.buildingId === featureId) {
                    matchingEntities.push(entity);
                    continue;
                  }

                  // Check _originalId property (set by GeoJsonData component)
                  if (props?._originalId === featureId) {
                    matchingEntities.push(entity);
                    continue;
                  }
                } catch {
                  // Silent fail for property access errors
                }
              }
            }
          }

          if (matchingEntities.length > 0) {
            // OPTIMIZED: Quick validation - only check first few positions per entity
            const validEntities = matchingEntities.filter((entity: any) => {
              try {
                if (entity.polygon?.hierarchy?.getValue) {
                  const hierarchy = entity.polygon.hierarchy.getValue();
                  if (hierarchy?.positions && hierarchy.positions.length > 0) {
                    // Sample validation: check first and last position only for speed
                    const firstPos = hierarchy.positions[0];
                    const lastPos =
                      hierarchy.positions[hierarchy.positions.length - 1];

                    const isValid = (pos: any) =>
                      pos &&
                      typeof pos.x === "number" &&
                      isFinite(pos.x) &&
                      typeof pos.y === "number" &&
                      isFinite(pos.y) &&
                      typeof pos.z === "number" &&
                      isFinite(pos.z);

                    return isValid(firstPos) && isValid(lastPos);
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

  const formattedData = useDataColumnizer({
    parsedData: selectedOutputData,
    type: fileType,
  });

  const featureIdMap = useMemo(() => {
    if (!formattedData.tableData) return null;

    const map = new Map<string | number, any>();
    formattedData.tableData.forEach((row: any) => {
      const id = row.id;
      const normalizedId = JSON.parse(id);
      map.set(normalizedId, row);
    });
    return map;
  }, [formattedData.tableData]);

  // Derive selectedFeature from selectedFeatureId
  const selectedFeature = useMemo(() => {
    if (!selectedFeatureId || !featureIdMap) return null;
    return featureIdMap.get(selectedFeatureId);
  }, [selectedFeatureId, featureIdMap]);

  const detailsFeature = useMemo(() => {
    if (!detailsOverlayOpen || !selectedFeature) return null;
    return selectedFeature;
  }, [detailsOverlayOpen, selectedFeature]);

  const handleFeatureSelect = useCallback(
    (featureId: string | null) => {
      if (selectedFeatureId !== featureId) {
        setSelectedFeatureId(featureId);
      }
    },
    [selectedFeatureId],
  );

  const handleRowSingleClick = useCallback(
    (value: any) => {
      // setEnableClustering(false);
      handleFeatureSelect(value?.id ?? null);
    },
    [handleFeatureSelect],
  );

  const handleRowDoubleClick = useCallback(
    (value: any) => {
      // setEnableClustering(false);
      const normalizedId = JSON.parse(value?.id);
      handleFeatureSelect(normalizedId ?? null);
      handleFlyToSelectedFeature(convertedSelectedFeature);
      setDetailsOverlayOpen(true);
    },
    [convertedSelectedFeature, handleFlyToSelectedFeature, handleFeatureSelect],
  );

  const handleCloseFeatureDetails = useCallback(() => {
    setDetailsOverlayOpen(false);
  }, []);

  const handleRemoveDataURL = useCallback(
    async (urlToRemove: string) => {
      if (!debugRunState || !currentProject?.id) return;

      const newDebugRunState = {
        ...debugRunState,
        jobs:
          debugRunState.jobs?.map((job) => {
            if (job.projectId !== currentProject.id) return job;

            const currentData = job.selectedIntermediateData ?? [];
            const filtered = currentData.filter(
              (sid) => sid.url !== urlToRemove,
            );

            return {
              ...job,
              selectedIntermediateData: filtered,
            };
          }) ?? [],
      };

      await updateValue(newDebugRunState);

      // check if the currently focused data URL was removed
      if (debugJobState?.focusedIntermediateData === urlToRemove) {
        await updateValue({
          ...newDebugRunState,
          jobs:
            newDebugRunState.jobs?.map((job, index) => {
              if (job.projectId !== currentProject.id) return job;

              const removedIndex = debugRunState.jobs?.[
                index
              ].selectedIntermediateData?.findIndex(
                (sid) => sid.url === urlToRemove,
              );

              // Try to focus on the next URL, or previous if last was removed
              let newFocusedURL: string | undefined = undefined;
              if (
                removedIndex !== undefined &&
                removedIndex >= 0 &&
                job.selectedIntermediateData &&
                job.selectedIntermediateData.length > 0
              ) {
                if (removedIndex < job.selectedIntermediateData.length) {
                  newFocusedURL =
                    job.selectedIntermediateData[removedIndex].url;
                } else if (removedIndex - 1 >= 0) {
                  newFocusedURL =
                    job.selectedIntermediateData[removedIndex - 1].url;
                }
              }

              return {
                ...job,
                focusedIntermediateData: newFocusedURL,
              };
            }) ?? [],
        });
        prevSelectedDataURLRef.current = undefined;
      }
    },
    [
      debugRunState,
      currentProject?.id,
      debugJobState?.focusedIntermediateData,
      updateValue,
    ],
  );

  return {
    debugJobId,
    debugJobState,
    fileType,
    mapRef,
    cesiumViewerRef,
    cesiumEntityMapRef,
    fullscreenDebug,
    expanded,
    minimized,
    selectedDataURL,
    dataURLs,
    outputDataForDownload,
    selectedOutputData,
    // enableClustering,
    selectedFeature,
    selectedFeatureId,
    detailsOverlayOpen,
    detailsFeature,
    formattedData,
    handleFeatureSelect,
    setConvertedSelectedFeature,
    // setEnableClustering,
    handleFullscreenExpand,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
    handleRemoveDataURL,
    handleRowSingleClick,
    handleRowDoubleClick,
    handleFlyToSelectedFeature,
    handleCloseFeatureDetails,

    // Data loading features (always available now)
    streamingQuery: streamingQuery,
    streamingProgress: streamingQuery.progress,
    detectedGeometryType: streamingQuery.detectedGeometryType,
    visualizerType: streamingQuery.visualizerType,
    totalFeatures: streamingQuery.totalFeatures,
    isComplete: streamingQuery.isComplete,
    isLoadingData: streamingQuery.isLoading || streamingQuery.isStreaming,
  };
};
