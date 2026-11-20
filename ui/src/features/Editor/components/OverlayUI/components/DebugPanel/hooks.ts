import type { MouseEvent } from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { featureRowsClient, useFeatureRows } from "@flow/lib/featureRows";
import type { FeatureTableRow } from "@flow/lib/featureRows";
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
import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";

export default () => {
  const [fullscreenDebug, setFullscreenDebug] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const prevSelectedDataURLRef = useRef<string | undefined>(undefined);

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

  // What "Download all" suggests calling the archive: the project's name, or
  // the job's id when the project has none.
  const archiveName = currentProject?.name || `job_${debugJobId}`;

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

  // The port the table shows: the focused one, or else the first.
  const metadataUrl =
    selectedDataURL ?? (dataURLs?.length ? dataURLs[0].key : "");

  // The port the table shows, as the API names it.
  const focusedFileId = useMemo(() => {
    const selection = debugJobState?.selectedIntermediateData?.find(
      (data) => data.url === metadataUrl,
    );
    return selection?.fileId ?? fileIdFromIntermediateDataUrl(metadataUrl);
  }, [debugJobState?.selectedIntermediateData, metadataUrl]);

  useEffect(() => {
    const client = featureRowsClient();
    return () => client.clear();
  }, [debugJobId]);

  const rows = useFeatureRows(metadataUrl);
  const { readLine } = rows;
  const loadedRowCount = rows.status?.rowCount ?? 0;
  const session = rows.status?.session ?? 0;

  const [selection, setSelection] = useState<{
    url: string;
    row: number;
  } | null>(null);
  const selectedRow = selection?.url === metadataUrl ? selection.row : null;
  const selectRow = useCallback(
    (row: number | null) =>
      setSelection(row === null ? null : { url: metadataUrl, row }),
    [metadataUrl],
  );

  const [loadedFeature, setLoadedFeature] = useState<{
    url: string;
    session: number;
    row: number;
    feature: any;
  } | null>(null);

  const selectedRowLoaded =
    selectedRow !== null && selectedRow < loadedRowCount;

  useEffect(() => {
    if (selectedRow === null || !selectedRowLoaded) return;
    let cancelled = false;
    void readLine(selectedRow).then((text) => {
      if (cancelled || text === null) return;
      const feature = toSelectedFeature(text, selectedRow);
      if (!feature) return;
      setLoadedFeature({
        url: metadataUrl,
        session,
        row: selectedRow,
        feature,
      });
    });
    return () => {
      cancelled = true;
    };
  }, [metadataUrl, readLine, selectedRow, selectedRowLoaded, session]);

  const selectedFeature =
    selectedRow !== null &&
    loadedFeature?.url === metadataUrl &&
    loadedFeature.session === session &&
    loadedFeature.row === selectedRow
      ? loadedFeature.feature
      : null;

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

  const featureRow: number | undefined = selectedFeature?._row;

  const modelViewRequest = useMemo(
    () =>
      debugJobId &&
      modelView &&
      modelView.fileId === focusedFileId &&
      modelView.row === featureRow
        ? { jobId: debugJobId, fileId: modelView.fileId, row: modelView.row }
        : undefined,
    [debugJobId, modelView, focusedFileId, featureRow],
  );

  // A 3D model needs 3D geometry somewhere on the globe; anything else would
  // only come back with nothing to show.
  const canOpenIn3D =
    !!debugJobId &&
    !!focusedFileId &&
    featureRow !== undefined &&
    !!selectedFeature?._values?.geometrySummary?.has3DWithCrs;

  const handleOpenIn3D = useCallback(() => {
    if (!debugJobId || !focusedFileId || featureRow === undefined) return;
    setModelView({ fileId: focusedFileId, row: featureRow });
    setModelViewOpenError(undefined);
    requestView({ jobId: debugJobId, fileId: focusedFileId, row: featureRow })
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
  }, [debugJobId, focusedFileId, featureRow, requestView]);

  const handleCloseModelView = useCallback(() => setModelView(null), []);

  // The port's full view, as tiles. Open for the port it was asked of.
  const [mapView, setMapView] = useState<{ fileId: string } | null>(null);
  const [mapViewOpenError, setMapViewOpenError] =
    useState<IntermediateDataViewError>();

  const mapViewRequest = useMemo(
    () =>
      debugJobId && mapView && mapView.fileId === focusedFileId
        ? { jobId: debugJobId, fileId: mapView.fileId }
        : undefined,
    [debugJobId, mapView, focusedFileId],
  );

  const canShowOnMap = !!debugJobId && !!focusedFileId;

  const handleShowOnMap = useCallback(() => {
    if (!debugJobId || !focusedFileId) return;
    setMapView({ fileId: focusedFileId });
    setMapViewOpenError(undefined);
    requestView({ jobId: debugJobId, fileId: focusedFileId }).catch(
      (err: unknown) => {
        if (
          err instanceof IntermediateDataViewError &&
          err.kind === "tooManyRenders"
        ) {
          setMapViewOpenError(err);
        }
      },
    );
  }, [debugJobId, focusedFileId, requestView]);

  const handleCloseMap = useCallback(() => setMapView(null), []);

  // Whether the last feature picked on the map is one the table has not
  // loaded, so its row cannot be shown. Cleared by the next pick or row click.
  const [pickedUnloadedRow, setPickedUnloadedRow] = useState(false);

  const handlePickRow = useCallback(
    (row: number | null) => {
      const loaded = row !== null && row < loadedRowCount;
      setPickedUnloadedRow(row !== null && !loaded);
      selectRow(loaded ? row : null);
    },
    [loadedRowCount, selectRow],
  );

  // Clicking the selected row again clears the selection, which also closes
  // the details pane.
  const handleRowSingleClick = useCallback(
    (row: FeatureTableRow) => {
      setPickedUnloadedRow(false);
      selectRow(row._row === selectedRow ? null : row._row);
    },
    [selectRow, selectedRow],
  );

  const handleRowDoubleClick = useCallback(
    (row: FeatureTableRow) => selectRow(row._row),
    [selectRow],
  );

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
    isDebugJobActive,
    fullscreenDebug,
    expanded,
    minimized,
    selectedDataURL,
    dataURLs,
    outputDataForDownload,
    archiveName,
    rows,
    selectedRow,
    selectedFeature,
    canOpenIn3D,
    modelViewRequest,
    modelViewOpenError,
    handleOpenIn3D,
    handleCloseModelView,
    canShowOnMap,
    mapViewRequest,
    mapViewOpenError,
    handleShowOnMap,
    handleCloseMap,
    handlePickRow,
    pickedUnloadedRow,
    loadedRowCount,
    handleFullscreenExpand,
    handleFullscreenExit,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
    handleRemoveDataURL,
    handleRowSingleClick,
    handleRowDoubleClick,
  };
};

export function toSelectedFeature(text: string, row: number) {
  let parsed: any;
  try {
    parsed = JSON.parse(text);
  } catch {
    return null;
  }
  let feature: any;
  try {
    feature = intermediateDataTransform(parsed);
  } catch (error) {
    console.warn("Failed to transform feature:", error);
    feature = { id: parsed.id, properties: parsed.attributes };
  }
  return {
    id: JSON.stringify(feature.id || row),
    _row: row,
    _values: {
      geometry: feature.geometry ?? {},
      attributes: feature.properties ?? {},
      geometrySummary: feature.geometrySummary,
    },
  };
}
