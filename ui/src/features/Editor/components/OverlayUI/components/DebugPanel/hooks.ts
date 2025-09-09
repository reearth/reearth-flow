import { useQuery } from "@tanstack/react-query";
import bbox from "@turf/bbox";
import { Cartesian3 } from "cesium";
import {
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import useFetchAndReadData from "@flow/hooks/useFetchAndReadData";
import { useStreamingDebugRunQuery } from "@flow/hooks/useStreamingDebugRunQuery";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

import { STREAMING_SIZE_THRESHOLD_MB } from "./constants";

export default () => {
  const t = useT();

  const prevIntermediateDataUrls = useRef<string[] | undefined>(undefined);
  const [fullscreenDebug, setFullscreenDebug] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [enableClustering, setEnableClustering] = useState<boolean>(true);
  const [selectedFeature, setSelectedFeature] = useState<any>(null);
  const [convertedSelectedFeature, setConvertedSelectedFeature] =
    useState(null);
  const mapRef = useRef<maplibregl.Map | null>(null);
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

  const [showTempPossibleIssuesDialog, setShowTempPossibleIssuesDialog] =
    useState(false);

  const { useGetJob } = useJob();

  const { job: debugJob, refetch } = useGetJob(debugJobState?.jobId ?? "");

  const outputURLs = useMemo(() => debugJob?.outputURLs, [debugJob]);

  const handleShowTempPossibleIssuesDialogClose = useCallback(() => {
    updateValue((prevState) => {
      const newJobs = prevState.jobs.map((pj) => {
        if (
          debugJob?.id === pj.jobId &&
          !pj.tempWorkflowHasPossibleIssuesFlag
        ) {
          return {
            ...pj,
            tempWorkflowHasPossibleIssuesFlag: false,
          };
        } else {
          return pj;
        }
      });
      return {
        jobs: newJobs,
      };
    });
    setShowTempPossibleIssuesDialog(false);
  }, [debugJob?.id, updateValue]);

  useEffect(() => {
    if (debugJobState?.tempWorkflowHasPossibleIssuesFlag) return;
    if (
      !outputURLs &&
      (debugJobState?.status === "completed" ||
        debugJobState?.status === "failed" ||
        debugJobState?.status === "cancelled")
    ) {
      (async () => {
        try {
          const { data: job } = await refetch();

          if (
            !job?.outputURLs &&
            debugJobState?.tempWorkflowHasPossibleIssuesFlag === undefined
          ) {
            updateValue((prevState) => {
              const newJobs = prevState.jobs.map((pj) => {
                if (
                  job?.id === pj.jobId &&
                  !pj.tempWorkflowHasPossibleIssuesFlag
                ) {
                  const tempFlag = !job.outputURLs?.length;
                  setShowTempPossibleIssuesDialog(tempFlag);
                  return {
                    ...pj,
                    tempWorkflowHasPossibleIssuesFlag: tempFlag, // No logsURL + a completed/failed/cancelled status means potential issues. @KaWaite
                  };
                } else {
                  return pj;
                }
              });
              return {
                jobs: newJobs,
              };
            });
          }
        } catch (error) {
          console.error("Error during refetch:", error);
        }
      })();
    }
  }, [
    debugJobState?.status,
    debugJobState?.tempWorkflowHasPossibleIssuesFlag,
    outputURLs,
    refetch,
    updateValue,
  ]);

  const intermediateDataURLs = useMemo(
    () => debugJobState?.selectedIntermediateData?.map((sid) => sid.url),
    [debugJobState],
  );

  const dataURLs = useMemo(() => {
    const urls: { key: string; name: string }[] = [];
    if (intermediateDataURLs) {
      intermediateDataURLs.forEach((intermediateDataURL) => {
        urls.push({
          key: intermediateDataURL,
          name: intermediateDataURL.split("/").pop() || intermediateDataURL,
        });
      });
    }
    if (outputURLs) {
      urls.push(
        ...outputURLs.map((url) => ({
          key: url,
          name: url.split("/").pop() + `(${t("Output data")})`,
        })),
      );
    }
    return urls.length ? urls : undefined;
  }, [outputURLs, intermediateDataURLs, t]);

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

  // Check file size first with a HEAD request
  const { data: fileMetadata } = useQuery({
    queryKey: ["fileMetadata", metadataUrl],
    queryFn: async () => {
      if (!metadataUrl) return null;

      const response = await fetch(metadataUrl, { method: "HEAD" });
      if (!response.ok) return null;

      return {
        contentLength: response.headers.get("content-length"),
        contentType: response.headers.get("content-type"),
      };
    },
    enabled: !!metadataUrl,
    staleTime: Infinity,
    gcTime: Infinity,
  });

  // Determine if we should use traditional loading based on data type and file size
  const shouldUseTraditionalLoading = useMemo(() => {
    const contentLength = fileMetadata?.contentLength;

    // Check if this is intermediate data (JSONL) vs output data
    const isIntermediateData = intermediateDataURLs?.includes(metadataUrl);
    const isOutputData = outputURLs?.includes(metadataUrl);

    // Only use streaming for JSONL intermediate data
    if (!isIntermediateData || isOutputData) {
      return true; // Use traditional for output data or non-intermediate data
    }

    // For intermediate JSONL data, use streaming by default since content-length is often missing
    if (!contentLength) {
      return false; // Default to streaming for JSONL when size unknown
    }

    const sizeInMB = parseInt(contentLength) / (1024 * 1024);
    const useTraditional = sizeInMB < STREAMING_SIZE_THRESHOLD_MB;

    return useTraditional; // Use traditional loading for files under 10MB
  }, [fileMetadata, metadataUrl, intermediateDataURLs, outputURLs]);

  // Use streaming query only for large files
  const streamingQuery = useStreamingDebugRunQuery(metadataUrl, {
    enabled: !!metadataUrl && !shouldUseTraditionalLoading,
  });

  // Use traditional fetch for small files or when streaming fails
  const {
    fileContent: traditionalData,
    fileType: traditionalFileType,
    isLoading: isLoadingTraditional,
  } = useFetchAndReadData({
    dataUrl: shouldUseTraditionalLoading
      ? (selectedDataURL ?? (dataURLs?.length ? dataURLs[0].key : ""))
      : "",
  });

  // Choose which data source to use
  const selectedOutputData = shouldUseTraditionalLoading
    ? traditionalData
    : streamingQuery.fileContent;
  const fileType = shouldUseTraditionalLoading
    ? traditionalFileType
    : streamingQuery.fileType;
  const isLoadingData = shouldUseTraditionalLoading
    ? isLoadingTraditional
    : streamingQuery.isLoading;

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

      // Get the current geometry type from streaming data or detect from selectedFeature
      let currentDetectedGeometryType = null;
      if (!shouldUseTraditionalLoading) {
        currentDetectedGeometryType = streamingQuery.detectedGeometryType;
      } else {
        // For traditional loading, detect from the selectedFeature geometry
        currentDetectedGeometryType = selectedFeature.geometry?.type;
      }

      // Determine which viewer to use based on detected geometry type
      const is3D = currentDetectedGeometryType === 'CityGmlGeometry' || currentDetectedGeometryType === 'FlowGeometry3D';

      if (is3D && cesiumViewerRef.current) {
        // 3D Cesium viewer - zoom to entities by feature ID
        try {
          const featureId = selectedFeature.id || selectedFeature.properties?._originalId;
          if (!featureId) {
            console.warn('No feature ID found for Cesium zoom');
            return;
          }

          // Find all entities that belong to this feature
          const matchingEntities = cesiumViewerRef.current.entities.values.filter((entity: any) => {
            const entityFeatureId = entity.properties?.getValue()?.buildingId || 
                                   entity.properties?.getValue()?.originalFeatureId ||
                                   entity.id?.split('_')[0]; // Extract base ID from compound IDs
            return entityFeatureId === featureId || 
                   JSON.stringify(entityFeatureId) === JSON.stringify(featureId);
          });

          if (matchingEntities.length > 0) {
            cesiumViewerRef.current.zoomTo(matchingEntities, {
              offset: new Cartesian3(0, 0, 100) // 100m offset from the building
            });
          } else {
            console.warn('No matching Cesium entities found for feature ID:', featureId);
          }
        } catch (err) {
          console.error('Error zooming to Cesium feature:', err);
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
    [shouldUseTraditionalLoading, streamingQuery.detectedGeometryType, cesiumViewerRef, mapRef],
  );

  const handleRowSingleClick = useCallback(
    (value: any) => {
      setEnableClustering(false);
      setSelectedFeature(value);
    },
    [setSelectedFeature, setEnableClustering],
  );

  const handleRowDoubleClick = useCallback(
    (value: any) => {
      setEnableClustering(false);
      setSelectedFeature(value);
      handleFlyToSelectedFeature(convertedSelectedFeature);
    },
    [
      convertedSelectedFeature,
      handleFlyToSelectedFeature,
      setSelectedFeature,
      setEnableClustering,
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
    showTempPossibleIssuesDialog,
    selectedDataURL,
    dataURLs,
    selectedOutputData,
    isLoadingData,
    enableClustering,
    selectedFeature,
    setSelectedFeature,
    setConvertedSelectedFeature,
    setEnableClustering,
    handleFullscreenExpand,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleShowTempPossibleIssuesDialogClose,
    handleSelectedDataChange,
    handleRowSingleClick,
    handleRowDoubleClick,
    handleFlyToSelectedFeature,

    // Streaming-specific features
    isStreaming: !shouldUseTraditionalLoading,
    streamingQuery: shouldUseTraditionalLoading ? null : streamingQuery,
    streamingProgress: shouldUseTraditionalLoading
      ? null
      : streamingQuery.progress,
    detectedGeometryType: shouldUseTraditionalLoading
      ? null
      : streamingQuery.detectedGeometryType,
    totalFeatures: shouldUseTraditionalLoading
      ? null
      : streamingQuery.totalFeatures,
    isComplete: shouldUseTraditionalLoading ? null : streamingQuery.isComplete,
  };
};
