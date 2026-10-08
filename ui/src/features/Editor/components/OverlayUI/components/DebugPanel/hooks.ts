import bbox from "@turf/bbox";
import {
  BoundingSphere,
  HeadingPitchRange,
  Math as CesiumMath,
  Rectangle,
} from "cesium";
import type { MouseEvent } from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { zoomToBoundingSphere } from "@flow/components/visualizations/Cesium/utils/cesiumFunctions";
import { isCityGmlGeometry } from "@flow/components/visualizations/Cesium/utils/cityGmlGeometryToPrimitives";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useStreamingDebugRunQuery } from "@flow/hooks/useStreamingDebugRunQuery";
import {
  IntermediateDataViewError,
  useIntermediateDataView,
} from "@flow/lib/gql/intermediateDataView";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { fileIdFromIntermediateDataUrl } from "@flow/lib/intermediateData";
import { useCurrentProject } from "@flow/stores";
import { toArtifactFiles } from "@flow/utils";

export default () => {
  const [fullscreenDebug, setFullscreenDebug] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [detailsOverlayOpen, setDetailsOverlayOpen] = useState(false);
  // The client-side map viewer, kept reachable while the rendered views
  // replace it. Off, the pane beside the table shows the selected row.
  const [legacyPreview, setLegacyPreview] = useState(false);
  const prevSelectedDataURLRef = useRef<string | undefined>(undefined);
  // const [enableClustering, setEnableClustering] = useState<boolean>(true);
  const [selectedFeatureId, setSelectedFeatureId] = useState<string | null>(
    null,
  );
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

  // Reads the same cache the debug action bar's subscription populates, so this
  // costs no extra socket. Only used to decide whether the diagnostics console
  // should poll — the status enum itself carries no diagnostics.
  const { data: realTimeJobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    debugJobId,
    !debugJobId,
  );

  const isDebugJobActive = useMemo(() => {
    const status = realTimeJobStatus ?? debugJob?.status;
    return status === "running" || status === "queued";
  }, [realTimeJobStatus, debugJob?.status]);

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

  // Separate output data for download functionality. The server hands the job's
  // artifacts over as a flat list of URLs, so the folder a writer with `groupBy`
  // set created is read back out of the URLs.
  const outputDataForDownload = useMemo(
    () => (outputURLs ? toArtifactFiles(outputURLs) : undefined),
    [outputURLs],
  );

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

  // The port the table shows, as the API names it.
  const focusedFileId = useMemo(() => {
    const selection = debugJobState?.selectedIntermediateData?.find(
      (data) => data.url === metadataUrl,
    );
    return selection?.fileId ?? fileIdFromIntermediateDataUrl(metadataUrl);
  }, [debugJobState?.selectedIntermediateData, metadataUrl]);

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

  const handleFullscreenExit = useCallback(() => setFullscreenDebug(false), []);

  const handleFlyToSelectedFeature = useCallback(
    (selectedFeature: any) => {
      if (!selectedFeature) return;

      // Which viewer is on screen, rather than which geometry type produced
      // it: the type is a display label that differs between the legacy and
      // new formats, while the viewer choice already encodes the dimension.
      const is3D =
        streamingQuery.visualizerType === "3d-map" ||
        streamingQuery.visualizerType === "3d-model";

      if (cesiumViewerRef.current) {
        const cesiumViewer = cesiumViewerRef.current?.cesiumElement;
        if (!cesiumViewer || cesiumViewer.isDestroyed()) return;
        if (is3D) {
          try {
            const featureId = selectedFeature.id;
            if (!featureId) return;

            const geometry = selectedFeature.geometry;

            // CityGML in either format is drawn as batched primitives, so
            // there is no entity for the entity search below to find; it has
            // to go through the bounding sphere. The check used to be on the
            // legacy type name, which a new-format feature does not carry.
            if (isCityGmlGeometry(geometry)) {
              zoomToBoundingSphere(geometry, cesiumViewerRef, 1.5);
            } else {
              // Non-CityGML 3D (e.g. FlowGeometry3D) — entity-based flyTo
              const entityValues = cesiumViewer?.entities?.values ?? [];
              const matchingEntities = entityValues.filter((entity: any) => {
                const props = entity.properties?.getValue?.();
                return (
                  props?._originalId === featureId || entity.id === featureId
                );
              });
              if (matchingEntities.length > 0) {
                cesiumViewer.zoomTo(matchingEntities);
              } else {
                // Search in data sources as fallback
                const dsCount = cesiumViewer.dataSources?.length ?? 0;
                for (let i = 0; i < dsCount; i++) {
                  const dataSource = cesiumViewer.dataSources.get(i);
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
    [streamingQuery.visualizerType, cesiumViewerRef],
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

  const selectedFeature = useMemo(
    () => findSelectedRow(featureIdMap, selectedFeatureId),
    [selectedFeatureId, featureIdMap],
  );

  const { requestView } = useIntermediateDataView();

  // The row a 3D model was opened for. The pane shows the model only while
  // that row is still the selected one; selecting another shows its details,
  // and the render goes on in the background.
  const [modelView, setModelView] = useState<{
    fileId: string;
    row: number;
  } | null>(null);
  const [modelViewOpenError, setModelViewOpenError] =
    useState<IntermediateDataViewError>();

  const selectedRow: number | undefined = selectedFeature?._row;

  const modelViewRequest = useMemo(
    () =>
      debugJobId &&
      modelView &&
      modelView.fileId === focusedFileId &&
      modelView.row === selectedRow
        ? { jobId: debugJobId, fileId: modelView.fileId, row: modelView.row }
        : undefined,
    [debugJobId, modelView, focusedFileId, selectedRow],
  );

  // A 3D model needs 3D geometry somewhere on the globe; anything else would
  // only come back with nothing to show.
  const canOpenIn3D =
    !!debugJobId &&
    !!focusedFileId &&
    selectedRow !== undefined &&
    !!selectedFeature?._values?.geometrySummary?.has3DWithCrs;

  const handleOpenIn3D = useCallback(() => {
    if (!debugJobId || !focusedFileId || selectedRow === undefined) return;
    setModelView({ fileId: focusedFileId, row: selectedRow });
    setModelViewOpenError(undefined);
    requestView({ jobId: debugJobId, fileId: focusedFileId, row: selectedRow })
      // A render that was asked for keeps its outcome in the view's own
      // state. Only one that could not be asked for at all needs keeping here.
      .catch((err: unknown) => {
        if (
          err instanceof IntermediateDataViewError &&
          err.kind === "tooManyRenders"
        ) {
          setModelViewOpenError(err);
        }
      });
  }, [debugJobId, focusedFileId, selectedRow, requestView]);

  const handleCloseModelView = useCallback(() => setModelView(null), []);

  const detailsFeature = useMemo(() => {
    if (!detailsOverlayOpen || !selectedFeature) return null;
    return selectedFeature;
  }, [detailsOverlayOpen, selectedFeature]);

  useEffect(() => {
    if (!selectedFeatureId || !featureIdMap) return;
    if (!findSelectedRow(featureIdMap, selectedFeatureId)) {
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

  // Clicking the selected row again clears the selection, which also closes
  // the details pane.
  const handleRowSingleClick = useCallback(
    (value: any) => {
      // setEnableClustering(false);
      handleFeatureSelect(
        selectionAfterRowClick(featureIdMap, selectedFeatureId, value),
      );
    },
    [handleFeatureSelect, featureIdMap, selectedFeatureId],
  );

  const handleRowDoubleClick = useCallback(
    (value: any) => {
      const normalizedId = JSON.parse(value?.id);
      handleFeatureSelect(normalizedId ?? null);
      // Without the map there is nothing to fly to, and the pane already
      // shows the row the overlay would.
      if (!legacyPreview) return;
      const feature =
        normalizedId != null
          ? (selectedOutputData?.features?.find(
              (f: any) => f.id === normalizedId,
            ) ?? null)
          : null;
      handleFlyToSelectedFeature(feature);
      setDetailsOverlayOpen(true);
    },
    [
      selectedOutputData,
      handleFlyToSelectedFeature,
      handleFeatureSelect,
      legacyPreview,
    ],
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
    isDebugJobActive,
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
    selectedFeature,
    legacyPreview,
    setLegacyPreview,
    canOpenIn3D,
    modelViewRequest,
    modelViewOpenError,
    handleOpenIn3D,
    handleCloseModelView,
    detailsOverlayOpen,
    detailsFeature,
    formattedData,
    handleFeatureSelect,
    // setEnableClustering,
    handleFullscreenExpand,
    handleFullscreenExit,
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

/**
 * The table row a selection names. A row click stores the row's serialized id
 * and a map click the parsed one, so both forms are looked up.
 */
export function findSelectedRow(
  rowsById: Map<string, any> | null,
  selectedId: string | null,
): any {
  if (!selectedId || !rowsById) return null;
  if (rowsById.has(selectedId)) return rowsById.get(selectedId);
  try {
    return rowsById.get(JSON.parse(selectedId)) ?? null;
  } catch {
    return null;
  }
}

/**
 * The selection after a row is clicked: the clicked row, or nothing when it
 * was the selected row already.
 */
export function selectionAfterRowClick(
  rowsById: Map<string, any> | null,
  selectedId: string | null,
  clicked: any,
): string | null {
  if (clicked == null) return null;
  if (findSelectedRow(rowsById, selectedId) === clicked) return null;
  return clicked.id ?? null;
}
