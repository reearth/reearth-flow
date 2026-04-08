import { useParams } from "@tanstack/react-router";
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

export default () => {
  const [fullscreenDebug, setFullscreenDebug] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [detailsOverlayOpen, setDetailsOverlayOpen] = useState(false);
  const prevSelectedDataURLRef = useRef<string | undefined>(undefined);
  const [selectedFeatureId, setSelectedFeatureId] = useState<string | null>(
    null,
  );
  const [convertedSelectedFeature, setConvertedSelectedFeature] =
    useState(null);
  const cesiumViewerRef = useRef<any>(null);

  const { debugId } = useParams({ strict: false }) as { debugId?: string };

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () => debugRunState?.jobs?.find((job) => job.jobId === debugId),
    [debugRunState, debugId],
  );

  const { useGetJob } = useJob();

  const { job: debugJob } = useGetJob(debugJobState?.jobId ?? "");

  const outputURLs = useMemo(() => debugJob?.outputURLs, [debugJob]);

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
    return urls.length ? urls : undefined;
  }, [debugJobState?.selectedIntermediateData]);

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
            if (job.jobId !== debugId) return job;
            return { ...job, focusedIntermediateData: url };
          }) ?? [],
      });
      setMinimized(false);
      prevSelectedDataURLRef.current = debugJobState?.focusedIntermediateData;
    }
  };

  const metadataUrl =
    selectedDataURL ?? (dataURLs?.length ? dataURLs[0].key : "");

  const streamingQuery = useStreamingDebugRunQuery(metadataUrl, {
    enabled: !!metadataUrl,
  });

  const selectedOutputData = streamingQuery.fileContent;
  const fileType = streamingQuery.fileType;

  const handleExpand = () => setExpanded((prev) => !prev);

  const handleMinimize = (e: MouseEvent) => {
    e.stopPropagation();
    setMinimized((prev) => !prev);
  };

  const handleTabChange = () => {
    if (minimized) setMinimized(false);
  };

  const handleFullscreenExpand = () => setFullscreenDebug((prev) => !prev);

  const handleFlyToSelectedFeature = useCallback(
    (selectedFeature: any) => {
      if (!selectedFeature) return;

      const currentDetectedGeometryType = streamingQuery.detectedGeometryType;
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
      handleFeatureSelect(value?.id ?? null);
    },
    [handleFeatureSelect],
  );

  const handleRowDoubleClick = useCallback(
    (value: any) => {
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
      if (!debugRunState || !debugId) return;

      const newDebugRunState = {
        ...debugRunState,
        jobs:
          debugRunState.jobs?.map((job) => {
            if (job.jobId !== debugId) return job;
            const currentData = job.selectedIntermediateData ?? [];
            return {
              ...job,
              selectedIntermediateData: currentData.filter(
                (sid) => sid.url !== urlToRemove,
              ),
            };
          }) ?? [],
      };

      await updateValue(newDebugRunState);

      if (debugJobState?.focusedIntermediateData === urlToRemove) {
        const remaining =
          debugJobState.selectedIntermediateData?.filter(
            (sid) => sid.url !== urlToRemove,
          ) ?? [];
        await updateValue({
          ...newDebugRunState,
          jobs: newDebugRunState.jobs?.map((job) => {
            if (job.jobId !== debugId) return job;
            return {
              ...job,
              focusedIntermediateData: remaining[0]?.url ?? undefined,
            };
          }),
        });
        prevSelectedDataURLRef.current = undefined;
      }
    },
    [debugRunState, debugId, debugJobState, updateValue],
  );

  return {
    debugJobId: debugId,
    debugJobState,
    cesiumViewerRef,
    fullscreenDebug,
    expanded,
    minimized,
    selectedDataURL,
    dataURLs,
    outputDataForDownload,
    selectedOutputData,
    selectedFeatureId,
    detailsOverlayOpen,
    detailsFeature,
    formattedData,
    handleFeatureSelect,
    setConvertedSelectedFeature,
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
    streamingQuery: streamingQuery,
    streamingProgress: streamingQuery.progress,
    detectedGeometryType: streamingQuery.detectedGeometryType,
    visualizerType: streamingQuery.visualizerType,
    totalFeatures: streamingQuery.totalFeatures,
    isComplete: streamingQuery.isComplete,
    isLoadingData: streamingQuery.isLoading || streamingQuery.isStreaming,
  };
};
