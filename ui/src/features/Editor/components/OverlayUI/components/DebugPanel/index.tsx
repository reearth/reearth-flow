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
import { DataTable } from "./DebugPreview/components";
import useHooks from "./hooks";

type Props = {
  fullscreenDebug: boolean;
  onFullScreenExpand: () => void;
};

const DebugPanel: React.FC<Props> = ({
  fullscreenDebug,
  onFullScreenExpand,
}) => {
  const {
    debugJobId,
    debugJobState,
    fileType,
    expanded,
    minimized,
    showTempPossibleIssuesDialog,
    selectedDataURL,
    dataURLs,
    selectedOutputData,
    isLoadingData,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleShowTempPossibleIssuesDialogClose,
    handleSelectedDataChange,
  } = useHooks();
  const t = useT();
  const [tabValue, setTabValue] = useState("debug-logs");
  const hasSwitchedToViewerRef = useRef(false);
  const debugJobIdRef = useRef(debugJobId);

  useEffect(() => {
    if (debugJobId !== debugJobIdRef.current) {
      debugJobIdRef.current = debugJobId;
    }

    if (dataURLs && !hasSwitchedToViewerRef.current) {
      setTabValue("debug-viewer");
      hasSwitchedToViewerRef.current = true;
    }

    if (!dataURLs) {
      setTabValue("debug-logs");
      hasSwitchedToViewerRef.current = false;
    }
  }, [dataURLs, debugJobId]);

  return debugJobId ? (
    <Tabs
      className={`pointer-events-auto w-[95vw] overflow-hidden rounded-md bg-secondary shadow-md shadow-secondary transition-all ${minimized ? "h-[36px]" : fullscreenDebug ? "h-[100vh] w-[100vw]" : expanded ? "h-[80vh]" : "h-[500px]"}`}
      value={tabValue}
      defaultValue="debug-logs"
      onDoubleClick={handleExpand}
      onValueChange={setTabValue}>
      <div className={`flex items-center ${minimized ? "" : "border-b"} p-1`}>
        <div className="relative flex w-fit items-center p-1">
          <div
            className={`flex w-fit items-center justify-start ${minimized ? "p-0" : "p-1"}`}>
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
        </div>
        <div className="mr-[120px] flex flex-1 items-center justify-center gap-2">
          <TerminalIcon />
          <p className="text-sm font-thin select-none">{t("Debug Run")}</p>
        </div>
        <div className="flex items-center gap-2">
          {!fullscreenDebug && (
            <div
              className="rounded p-1 hover:bg-primary"
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
              className="rounded p-1 hover:bg-primary"
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
              className="rounded p-1 hover:bg-primary"
              onClick={onFullScreenExpand}>
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
        <DebugLogs debugJobId={debugJobId} />{" "}
      </TabsContent>
      {dataURLs && (
        <TabsContent
          value="debug-viewer"
          forceMount={true}
          hidden={tabValue !== "debug-viewer"}
          className="h-[calc(100%-35px)] overflow-scroll">
          <ResizablePanelGroup className="h-full w-full" direction="horizontal">
            <ResizablePanel defaultSize={70} minSize={20}>
              <Tabs defaultValue="data-viewer">
                <div className="relative flex w-fit items-center p-1">
                  <div className="flex w-fit items-center justify-start p-1">
                    <div className="top-1 left-1">
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
                  </div>
                </div>
              </Tabs>
              <DataTable fileContent={selectedOutputData} fileType={fileType} />
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel defaultSize={30} minSize={20}>
              <DebugPreview
                debugJobState={debugJobState}
                dataURLs={dataURLs}
                fileType={fileType}
                selectedOutputData={selectedOutputData}
                isLoadingData={isLoadingData}
                showTempPossibleIssuesDialog={showTempPossibleIssuesDialog}
                onShowTempPossibleIssuesDialogClose={
                  handleShowTempPossibleIssuesDialogClose
                }
              />
            </ResizablePanel>
          </ResizablePanelGroup>
        </TabsContent>
      )}
    </Tabs>
  ) : null;
};

export default memo(DebugPanel);
