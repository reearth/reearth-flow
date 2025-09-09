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

import { useStreamingDebugRunQuery } from "@flow/hooks/useStreamingDebugRunQuery";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

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

  const streamingQuery = useStreamingDebugRunQuery(metadataUrl, {
    enabled: !!metadataUrl,
  });

  const selectedOutputData = streamingQuery.fileContent;
  const fileType = streamingQuery.fileType;
  const isLoadingData = streamingQuery.isLoading;

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
      const is3D = currentDetectedGeometryType === 'CityGmlGeometry' || currentDetectedGeometryType === 'FlowGeometry3D';

      if (is3D && cesiumViewerRef.current) {
        // 3D Cesium viewer - zoom to entities by feature ID
        try {
          // Access the actual Cesium viewer from Resium component
          const cesiumViewer = cesiumViewerRef.current?.cesiumElement;
          
          if (!cesiumViewer) {
            console.warn('Cesium viewer not initialized yet');
            return;
          }
          
          const featureId = selectedFeature.id || selectedFeature.properties?._originalId;
          if (!featureId) {
            console.warn('No feature ID found for Cesium zoom');
            return;
          }

          // Safety check for entities collection
          if (!cesiumViewer.entities || !cesiumViewer.entities.values) {
            console.warn('Cesium entities collection not available yet');
            return;
          }

          // Find all entities that belong to this feature
          const matchingEntities = cesiumViewer.entities.values.filter((entity: any) => {
            // Check direct entity ID match (main building entity)
            if (entity.id === featureId || JSON.stringify(entity.id) === JSON.stringify(featureId)) {
              return true;
            }
            
            // Check buildingId property (surface entities)
            const buildingId = entity.properties?.getValue()?.buildingId;
            if (buildingId && (buildingId === featureId || JSON.stringify(buildingId) === JSON.stringify(featureId))) {
              return true;
            }
            
            // Check compound ID prefix (surface entities like "buildingId_wall_1")
            if (entity.id && typeof entity.id === 'string' && entity.id.includes('_')) {
              const baseId = entity.id.split('_')[0];
              if (baseId === featureId || JSON.stringify(baseId) === JSON.stringify(featureId)) {
                return true;
              }
            }
            
            return false;
          });

          if (matchingEntities.length > 0) {
            // Validate entities have reasonable coordinates
            const validEntities = matchingEntities.filter((entity: any) => {
              try {
                if (entity.polygon?.hierarchy?.getValue) {
                  const hierarchy = entity.polygon.hierarchy.getValue();
                  if (hierarchy?.positions) {
                    // Check if any position has invalid coordinates
                    return hierarchy.positions.every((pos: any) => 
                      pos && 
                      typeof pos.x === 'number' && !isNaN(pos.x) && isFinite(pos.x) &&
                      typeof pos.y === 'number' && !isNaN(pos.y) && isFinite(pos.y) &&
                      typeof pos.z === 'number' && !isNaN(pos.z) && isFinite(pos.z)
                    );
                  }
                }
                return true; // If no polygon, assume valid
              } catch {
                return false;
              }
            });
            
            if (validEntities.length === 0) {
              console.warn('No valid entities found - all have invalid coordinates');
              return;
            }
            
            try {
              // Try different zoom approaches to handle potential coordinate issues
              
              // Approach 1: Simple zoomTo without offset on valid entities
              cesiumViewer.zoomTo(validEntities);
              
            } catch (zoomError) {
              console.warn('Direct zoomTo failed, trying fallback approach:', zoomError);
              
              try {
                // Approach 2: Zoom to first valid entity only
                if (validEntities[0]) {
                  cesiumViewer.zoomTo(validEntities[0]);
                }
              } catch (fallbackError) {
                console.error('All zoom approaches failed:', fallbackError);
                
                // Approach 3: Manual camera positioning using entity bounds
                try {
                  const entity = validEntities[0];
                  if (entity.position) {
                    const position = entity.position.getValue();
                    if (position) {
                      cesiumViewer.camera.lookAt(
                        position,
                        new Cartesian3(100, 100, 100) // Simple offset
                      );
                    }
                  }
                } catch (manualError) {
                  console.error('Manual camera positioning failed:', manualError);
                }
              }
            }
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
    [streamingQuery.detectedGeometryType, cesiumViewerRef, mapRef],
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

    // Streaming features (always available now)
    isStreaming: true,
    streamingQuery: streamingQuery,
    streamingProgress: streamingQuery.progress,
    detectedGeometryType: streamingQuery.detectedGeometryType,
    totalFeatures: streamingQuery.totalFeatures,
    isComplete: streamingQuery.isComplete,
  };
};
