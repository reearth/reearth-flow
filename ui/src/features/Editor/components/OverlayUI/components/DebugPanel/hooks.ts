import bbox from "@turf/bbox";
import {
  BoundingSphere,
  HeadingPitchRange,
  Math as CesiumMath,
  Rectangle,
} from "cesium";
import {
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { zoomToBoundingSphere } from "@flow/components/visualizations/Cesium/utils/cesiumFunctions";
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
  const cesiumViewerRef = useRef<any>(null);

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
      name: decodeURIComponent(url.split("/").pop() || url),
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

      if (cesiumViewerRef.current) {
        const cesiumViewer = cesiumViewerRef.current?.cesiumElement;
        if (is3D) {
          try {
            const featureId = selectedFeature.id;
            if (!featureId) return;

            const geometry = selectedFeature.geometry;

            if (geometry?.type === "CityGmlGeometry") {
              zoomToBoundingSphere(geometry, cesiumViewerRef, 1.5);
            } else {
              // Non-CityGML 3D (e.g. FlowGeometry3D) — entity-based flyTo
              const matchingEntities =
                cesiumViewerRef.current?.cesiumElement.entities.values.filter(
                  (entity: any) => {
                    const props = entity.properties?.getValue?.();
                    return (
                      props?._originalId === featureId ||
                      entity.id === featureId
                    );
                  },
                );
              if (matchingEntities.length > 0) {
                cesiumViewerRef.current?.cesiumElement.zoomTo(matchingEntities);
              } else {
                // Search in data sources as fallback
                for (const dataSource of cesiumViewer.dataSources) {
                  const matching = dataSource.entities.values.filter(
                    (entity: any) => {
                      const props = entity.properties?.getValue?.();
                      return (
                        props?._originalId === featureId ||
                        entity.id === featureId
                      );
                    },
                  );
                  if (matching.length > 0) {
                    cesiumViewer.zoomTo(matching);
                    break;
                  }
                }
              }
            }
          } catch (err) {
            console.error("Error zooming to Cesium feature:", err);
          }
        } else {
          try {
            const [minLng, minLat, maxLng, maxLat] = bbox(selectedFeature);

            const rect = Rectangle.fromDegrees(minLng, minLat, maxLng, maxLat);
            const sphere = BoundingSphere.fromRectangle3D(rect);
            const paddedSphere = new BoundingSphere(
              sphere.center,
              Math.max(sphere.radius * 1.5, 500),
            );

            cesiumViewer.camera.flyToBoundingSphere(paddedSphere, {
              duration: 1.5,
              offset: new HeadingPitchRange(
                0,
                CesiumMath.toRadians(-90),
                paddedSphere.radius * 2,
              ),
            });
          } catch (error) {
            console.error("Error calculating bounding box for feature:", error);
          }
        }
      }
    },
    [streamingQuery.detectedGeometryType, cesiumViewerRef],
  );

  const formattedData = useDataColumnizer({
    parsedData: selectedOutputData,
    type: fileType,
  });

  const featureIdMap = useMemo(() => {
    if (!formattedData.tableData) return null;

    const map = new Map<string, any>();
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

  useEffect(() => {
    if (!selectedFeatureId || !featureIdMap) return;
    if (!featureIdMap.has(selectedFeatureId)) {
      setSelectedFeatureId(null);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [featureIdMap]);

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

  const handleShowFeatureDetailsOverlay = useCallback((value: boolean) => {
    setDetailsOverlayOpen(value);
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
    cesiumViewerRef,
    fullscreenDebug,
    expanded,
    minimized,
    selectedDataURL,
    dataURLs,
    outputDataForDownload,
    selectedOutputData,
    // enableClustering,
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
    handleShowFeatureDetailsOverlay,

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
