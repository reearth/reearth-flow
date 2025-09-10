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
import { memo, useEffect, useRef, useState } from "react";

import {
  IconButton,
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
import TableViewer from "./DebugPreview/components/TableViewer";
import useHooks from "./hooks";
import OutputDataDownload from "./OutputDataDownload";

const DebugPanel: React.FC = () => {
  const {
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
    outputDataForDownload,
    selectedOutputData,
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
    // Data properties
    detectedGeometryType,
    totalFeatures,
    isComplete,
  } = useHooks();
  const t = useT();
  const [tabValue, setTabValue] = useState("debug-logs");

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
      className={`${fullscreenDebug ? "fixed inset-0" : ""} z-30 flex items-end`}>
      <Tabs
        className={`pointer-events-auto w-[95vw] rounded-md border border-primary bg-secondary/70 p-2 shadow-md shadow-secondary backdrop-blur transition-all ${minimized ? "h-[42px]" : fullscreenDebug ? "h-[100vh] w-[100vw]" : expanded ? "h-[75vh]" : "h-[45vh]"}`}
        value={tabValue}
        defaultValue="debug-logs"
        onValueChange={setTabValue}>
        <div className="flex justify-between pb-2" onDoubleClick={handleExpand}>
          <div className="flex w-fit items-center">
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
          <div className="mr-[120px] flex items-center justify-center gap-2">
            <TerminalIcon />
            <p className="text-sm font-thin select-none">{t("Debug Run")}</p>
          </div>
          <div className="flex items-center">
            {!fullscreenDebug && (
              <IconButton
                className="h-[25px] cursor-pointer rounded hover:bg-primary"
                icon={
                  minimized ? (
                    <CaretUpIcon weight="light" />
                  ) : (
                    <MinusIcon weight="light" />
                  )
                }
                onClick={handleMinimize}
              />
            )}
            {!minimized && !fullscreenDebug && (
              <IconButton
                className="h-[25px] cursor-pointer rounded hover:bg-primary"
                icon={
                  expanded ? (
                    <CaretDownIcon weight="light" />
                  ) : (
                    <CaretUpIcon weight="light" />
                  )
                }
                onClick={handleExpand}
              />
            )}
            {!minimized && (
              <IconButton
                className="h-[25px] cursor-pointer rounded hover:bg-primary"
                icon={
                  fullscreenDebug ? (
                    <CornersInIcon weight="light" />
                  ) : (
                    <CornersOutIcon weight="light" />
                  )
                }
                onClick={handleFullscreenExpand}
              />
            )}
          </div>
        </div>
        <TabsContent
          value="debug-logs"
          className="h-[calc(100%-30px)] overflow-scroll"
          forceMount={debugJobIdRef.current !== debugJobId ? undefined : true}
          hidden={tabValue !== "debug-logs"}>
          <DebugLogs debugJobId={debugJobId} />
        </TabsContent>
        {dataURLs && (
          <TabsContent
            value="debug-viewer"
            forceMount={true}
            hidden={tabValue !== "debug-viewer"}
            className="h-[calc(100%-30px)] overflow-scroll">
            <ResizablePanelGroup direction="horizontal">
              <ResizablePanel
                defaultSize={60}
                minSize={20}
                className="flex flex-col">
                  <div className="flex gap-2 py-2">
                    <Select
                      defaultValue={dataURLs[0].key}
                      value={selectedDataURL}
                      onValueChange={handleSelectedDataChange}>
                      <SelectTrigger className="h-[26px] w-auto max-w-[250px] text-xs font-bold">
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
                    <OutputDataDownload outputData={outputDataForDownload} />
                  </div>
                <div className="min-h-0 flex-1">
                  <TableViewer
                    fileContent={selectedOutputData}
                    fileType={fileType}
                    selectedFeature={selectedFeature}
                    onSingleClick={handleRowSingleClick}
                    onDoubleClick={handleRowDoubleClick}
                    detectedGeometryType={detectedGeometryType || undefined}
                    totalFeatures={totalFeatures || undefined}
                  />
                </div>
              </ResizablePanel>
              <ResizableHandle className="data-resize-handle-[state=drag]:border-logo/70 mx-2 h-[30%] w-1 self-center rounded-md border border-accent bg-accent transition hover:border-transparent hover:bg-logo/70" />
              <ResizablePanel defaultSize={40} minSize={20}>
                <DebugPreview
                  debugJobState={debugJobState}
                  dataURLs={dataURLs}
                  fileType={fileType}
                  selectedOutputData={selectedOutputData}
                  showTempPossibleIssuesDialog={showTempPossibleIssuesDialog}
                  selectedFeature={selectedFeature}
                  enableClustering={enableClustering}
                  mapRef={mapRef}
                  cesiumViewerRef={cesiumViewerRef}
                  onConvertedSelectedFeature={setConvertedSelectedFeature}
                  onShowTempPossibleIssuesDialogClose={
                    handleShowTempPossibleIssuesDialogClose
                  }
                  onSelectedFeature={setSelectedFeature}
                  onEnableClusteringChange={setEnableClustering}
                  onFlyToSelectedFeature={handleFlyToSelectedFeature}
                  // Data detection props
                  detectedGeometryType={detectedGeometryType}
                  isComplete={isComplete}
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
