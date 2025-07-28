import {
  CaretDownIcon,
  CaretUpIcon,
  CodeIcon,
  CornersInIcon,
  CornersOutIcon,
  EyeIcon,
  MinusIcon,
  TerminalIcon,
} from "@phosphor-icons/react";
import bbox from "@turf/bbox";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import DebugLogs from "./DebugLogs";
import DebugPreview from "./DebugPreview";
import DataTable from "./DebugPreview/components/DataTable";
import useHooks from "./hooks";

const DebugPanel: React.FC = () => {
  const {
    debugJobId,
    debugJobState,
    fileType,
    mapRef,
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
    setEnableClustering,
    handleFullscreenExpand,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleShowTempPossibleIssuesDialogClose,
    handleSelectedDataChange,
  } = useHooks();
  const t = useT();
  const [tabValue, setTabValue] = useState("debug-logs");

  const { featureMap, processedOutputData } = useMemo(() => {
    if (!selectedOutputData?.features) {
      return { featureMap: null, processedOutputData: selectedOutputData };
    }

    const map = new Map<string | number, any>();
    const processedFeatures = selectedOutputData.features.map((f: any) => {
      const processedFeature = {
        ...f,
        properties: {
          _originalId: f.id,
          ...f.properties,
        },
      };

      if (f.id !== undefined) {
        map.set(f.id, processedFeature);
      }

      return processedFeature;
    });

    return {
      featureMap: map,
      processedOutputData: {
        ...selectedOutputData,
        features: processedFeatures,
      },
    };
  }, [selectedOutputData]);

  const convertFeature = useCallback(
    (feature: any) => {
      if (!feature || !featureMap) return null;

      if ("geometry" in feature && feature.geometry) {
        return feature;
      }

      const featureId = feature.properties?._originalId ?? feature.id;
      if (featureId === undefined) return null;

      let normalizedId = featureId;
      if (typeof featureId === "string") {
        try {
          normalizedId = JSON.parse(featureId);
        } catch {
          normalizedId = featureId;
        }
      }

      return featureMap.get(normalizedId) || null;
    },
    [featureMap],
  );

  const convertedSelectedFeature = useMemo(() => {
    return convertFeature(selectedFeature);
  }, [selectedFeature, convertFeature]);

  const handleFlyToSelectedFeature = useCallback(
    (selectedFeature: any) => {
      if (mapRef.current && selectedFeature) {
        try {
          const [minLng, minLat, maxLng, maxLat] = bbox(selectedFeature);
          mapRef.current.fitBounds(
            [
              [minLng, minLat],
              [maxLng, maxLat],
            ],

            { padding: 40, duration: 500, maxZoom: 24 },
          );
        } catch (err) {
          console.error("Error computing bbox for selectedFeature:", err);
        }
      }
    },
    [mapRef],
  );
  const handleSingleClick = useCallback(
    (value: any) => {
      if (selectedFeature && value && selectedFeature.id === value.id) {
        setSelectedFeature(null);
      } else {
        setEnableClustering(false);
        setSelectedFeature(value);
      }
    },
    [selectedFeature, setSelectedFeature, setEnableClustering],
  );

  const handleDoubleClick = useCallback(
    (value: any) => {
      setEnableClustering(false);
      setSelectedFeature(value);
      handleFlyToSelectedFeature(convertFeature(value));
    },
    [
      handleFlyToSelectedFeature,
      setSelectedFeature,
      setEnableClustering,
      convertFeature,
    ],
  );

  const hasSwitchedToViewerRef = useRef(false);
  const debugJobIdRef = useRef(debugJobId);

  useEffect(() => {
    if (debugJobId !== debugJobIdRef.current) {
      debugJobIdRef.current = debugJobId;
    }
  }, [debugJobId]);

  useEffect(() => {
    if (dataURLs && !hasSwitchedToViewerRef.current) {
      setTabValue("debug-viewer");
      hasSwitchedToViewerRef.current = true;
    }

    if (!dataURLs) {
      setTabValue("debug-logs");
      hasSwitchedToViewerRef.current = false;
    }
  }, [dataURLs]);

  return debugJobId ? (
    <div
      className={`absolute ${fullscreenDebug ? "bottom-0 left-0" : "bottom-4 left-4 "}  z-30 flex items-end`}>
      <Tabs
        className={`pointer-events-auto w-[95vw] rounded-md bg-secondary shadow-md shadow-secondary transition-all ${minimized ? "h-[38px]" : fullscreenDebug ? "h-[100vh] w-[100vw]" : expanded ? "h-[80vh]" : "h-[500px]"}`}
        value={tabValue}
        defaultValue="debug-logs"
        onValueChange={setTabValue}>
        <div
          className={`flex ${minimized ? "" : "border-b"} p-1`}
          onDoubleClick={handleExpand}>
          <div className="flex w-fit items-center justify-start p-1">
            <TabsList className="gap-2">
              <TabsTrigger
                className="gap-1 bg-card font-thin"
                value="debug-logs"
                onClick={handleTabChange}>
                <CodeIcon />
                <p className="text-sm font-thin select-none">
                  {t("Workflow Logs")}
                </p>
              </TabsTrigger>
              <TabsTrigger
                className="gap-1 bg-card font-thin"
                value="debug-viewer"
                disabled={!dataURLs?.length}
                onClick={handleTabChange}>
                <EyeIcon />
                <p className="text-sm font-thin select-none">
                  {t("Data Preview")}
                </p>
              </TabsTrigger>
            </TabsList>
          </div>
          <div className="mr-[120px] flex flex-1 items-center justify-center gap-2">
            <TerminalIcon />
            <p className="text-sm font-thin select-none">{t("Debug Run")}</p>
          </div>
          <div className="flex items-center gap-2 p-1">
            {!fullscreenDebug && (
              <div
                className=" rounded p-1 hover:bg-primary"
                onClick={handleMinimize}>
                {minimized ? (
                  <CaretUpIcon weight="light" />
                ) : (
                  <MinusIcon weight="light" />
                )}
              </div>
            )}
            {!minimized && !fullscreenDebug && (
              <div
                className="cursor-pointer rounded p-1 hover:bg-primary"
                onClick={handleExpand}>
                {expanded ? (
                  <CaretDownIcon weight="light" />
                ) : (
                  <CaretUpIcon weight="light" />
                )}
              </div>
            )}
            {!minimized && (
              <div
                className="cursor-pointer rounded p-1 hover:bg-primary"
                onClick={handleFullscreenExpand}>
                {fullscreenDebug ? (
                  <CornersInIcon weight="light" />
                ) : (
                  <CornersOutIcon weight="light" />
                )}
              </div>
            )}
          </div>
        </div>
        <TabsContent
          value="debug-logs"
          className="h-[calc(100%-35px)] overflow-scroll"
          forceMount={debugJobIdRef.current !== debugJobId ? undefined : true}
          hidden={tabValue !== "debug-logs"}>
          <DebugLogs debugJobId={debugJobId} />
        </TabsContent>
        {dataURLs && (
          <TabsContent
            value="debug-viewer"
            forceMount={true}
            hidden={tabValue !== "debug-viewer"}
            className="h-[calc(100%-35px)] overflow-scroll">
            <ResizablePanelGroup direction="horizontal">
              <ResizablePanel defaultSize={70} minSize={20}>
                <Tabs defaultValue="data-viewer">
                  <div className="top-1 left-1 p-1 pt-2">
                    <Select
                      defaultValue={dataURLs[0].key}
                      value={selectedDataURL}
                      onValueChange={handleSelectedDataChange}>
                      <SelectTrigger className="h-[26px] max-w-[200px] border-none text-xs font-bold">
                        <SelectValue
                          placeholder={t("Select Data to Preview")}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {dataURLs.map(({ key, name }) => (
                          <SelectItem key={key} value={key}>
                            {name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </Tabs>
                <DataTable
                  fileContent={selectedOutputData}
                  fileType={fileType}
                  selectedFeature={selectedFeature}
                  onSingleClick={handleSingleClick}
                  onDoubleClick={handleDoubleClick}
                />
              </ResizablePanel>
              <ResizableHandle className="data-resize-handle-[state=drag]:border-logo/70 relative m-[0px] border border-border/50 transition hover:border-logo/70" />
              <ResizablePanel defaultSize={30} minSize={20}>
                <DebugPreview
                  debugJobState={debugJobState}
                  dataURLs={dataURLs}
                  fileType={fileType}
                  selectedOutputData={processedOutputData}
                  isLoadingData={isLoadingData}
                  showTempPossibleIssuesDialog={showTempPossibleIssuesDialog}
                  selectedFeature={selectedFeature}
                  convertedSelectedFeature={convertedSelectedFeature}
                  enableClustering={enableClustering}
                  mapRef={mapRef}
                  onShowTempPossibleIssuesDialogClose={
                    handleShowTempPossibleIssuesDialogClose
                  }
                  onSelectedFeature={setSelectedFeature}
                  onEnableClusteringChange={setEnableClustering}
                  onFlyToSelectedFeature={handleFlyToSelectedFeature}
                />
              </ResizablePanel>
            </ResizablePanelGroup>
          </TabsContent>
        )}
      </Tabs>
    </div>
  ) : null;
};

export default memo(DebugPanel);
