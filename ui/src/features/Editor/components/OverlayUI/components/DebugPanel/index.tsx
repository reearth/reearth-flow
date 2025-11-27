import {
  CaretDownIcon,
  CaretUpIcon,
  CodeIcon,
  CornersInIcon,
  CornersOutIcon,
  EyeIcon,
  MinusIcon,
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
    selectedDataURL,
    dataURLs,
    outputDataForDownload,
    selectedOutputData,
    selectedFeature,
    setConvertedSelectedFeature,
    handleSelectedFeature,
    handleFullscreenExpand,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
    handleRowSingleClick,
    handleRowDoubleClick,
    handleFlyToSelectedFeature,
    // Data properties
    detectedGeometryType,
    visualizerType,
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
        className={`pointer-events-auto border border-primary bg-secondary/70 p-1 shadow-md shadow-secondary backdrop-blur transition-all ${minimized ? "h-[42px] w-[96vw] rounded-xl" : fullscreenDebug ? "h-[100vh] w-[100vw] rounded-none" : expanded ? "h-[65vh] w-[99vw] rounded-xl" : "h-[45vh] w-[96vw] rounded-xl"}`}
        value={tabValue}
        defaultValue="debug-logs"
        onValueChange={setTabValue}>
        <div
          className="relative flex justify-between"
          onDoubleClick={handleExpand}>
          <div className="flex w-fit items-center">
            <TabsList className="gap-2">
              <TabsTrigger
                className="group h-8 gap-1 border border-transparent bg-card font-thin data-[state=active]:border-logo/40"
                value="debug-logs"
                onClick={handleTabChange}>
                <CodeIcon className="group-data-[state=active]:fill-logo" />
                <p className="text-sm font-thin select-none">
                  {t("Workflow Logs")}
                </p>
              </TabsTrigger>
              <TabsTrigger
                className="group h-8 gap-1 border border-transparent bg-card font-thin data-[state=active]:border-logo/40"
                value="debug-viewer"
                disabled={!dataURLs?.length}
                onClick={handleTabChange}>
                <EyeIcon className="group-data-[state=active]:fill-logo" />
                <p className="text-sm font-thin select-none">
                  {t("Data Preview")}
                </p>
              </TabsTrigger>
            </TabsList>
            <div className="ml-2 h-full w-1 border-l" />
            <OutputDataDownload outputData={outputDataForDownload} />
          </div>
          {/* <div className="absolute left-1/2 mr-[120px] flex h-full translate-x-1/2 items-center justify-center gap-2">
            <TerminalIcon />
            <p className="text-sm font-thin select-none">{t("Debug Run")}</p>
          </div> */}
          <div className="flex items-center">
            {!fullscreenDebug && (
              <IconButton
                className="h-8 cursor-pointer rounded hover:bg-primary"
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
                className="h-8 cursor-pointer rounded hover:bg-primary"
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
                className="h-8 cursor-pointer rounded hover:bg-primary"
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
            className="h-[calc(100%-32px)] overflow-scroll">
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
                    <SelectTrigger className="h-[26px] w-auto max-w-[300px] text-xs font-bold">
                      <SelectValue placeholder={t("Select Data to Preview")} />
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
              {!minimized && (
                <ResizableHandle className="data-resize-handle-[state=drag]:border-logo/70 mx-2 h-[30%] w-1 self-center rounded-md border border-accent bg-accent transition hover:border-transparent hover:bg-logo/70" />
              )}
              <ResizablePanel defaultSize={40} minSize={20}>
                <DebugPreview
                  debugJobState={debugJobState}
                  dataURLs={dataURLs}
                  fileType={fileType}
                  selectedOutputData={selectedOutputData}
                  selectedFeature={selectedFeature}
                  mapRef={mapRef}
                  cesiumViewerRef={cesiumViewerRef}
                  onConvertedSelectedFeature={setConvertedSelectedFeature}
                  onSelectedFeature={handleSelectedFeature}
                  onFlyToSelectedFeature={handleFlyToSelectedFeature}
                  // Data detection props
                  detectedGeometryType={detectedGeometryType}
                  visualizerType={visualizerType}
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
