import {
  CaretDownIcon,
  CaretUpIcon,
  CodeIcon,
  EyeIcon,
  GridNineIcon,
  MinusIcon,
  TerminalIcon,
} from "@phosphor-icons/react";
import { memo } from "react";

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

const DebugPanel: React.FC = () => {
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
  return debugJobId ? (
    <div
      className={`pointer-events-auto w-[90vw] cursor-pointer rounded-md bg-secondary shadow-md shadow-secondary transition-all ${minimized ? "h-[24px]" : expanded ? "h-[90vh]" : "h-[500px]"}`}>
      <div className="flex items-center border-b p-1" onClick={handleExpand}>
        <div className="flex flex-1 items-center justify-center gap-2 ">
          <TerminalIcon />
          <p className="text-sm font-thin select-none">{t("Debug Run")}</p>
        </div>
        <div className="flex items-center gap-2">
          <div
            className="rounded p-1 hover:bg-primary"
            onClick={handleMinimize}>
            {minimized ? (
              <CaretUpIcon weight="light" />
            ) : (
              <MinusIcon weight="light" />
            )}
          </div>
          {!minimized && (
            <div className="rounded p-1 hover:bg-primary">
              {expanded ? (
                <CaretDownIcon weight="light" />
              ) : (
                <CaretUpIcon weight="light" />
              )}
            </div>
          )}
        </div>
      </div>
      <Tabs
        className={`pointer-events-auto h-full w-full rounded-md bg-secondary p-1 shadow-md shadow-secondary transition-all`}
        defaultValue="debug-logs">
        <div className="relative flex items-center p-1">
          <div className="flex w-full items-center justify-start p-1">
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
        <TabsContent
          className="h-[calc(100%-35px)] overflow-scroll"
          value="debug-logs">
          <DebugLogs />{" "}
        </TabsContent>
        {dataURLs && (
          <TabsContent
            className="h-[calc(100%-35px)] overflow-scroll"
            value="debug-viewer">
            <ResizablePanelGroup
              className="h-full w-full"
              direction="horizontal">
              <ResizablePanel defaultSize={70} minSize={20}>
                <Tabs defaultValue="data-viewer">
                  <div className="relative flex items-center p-1">
                    <div className="flex w-full items-center justify-start p-1">
                      <TabsList className="gap-2">
                        <TabsTrigger
                          className="gap-1 bg-card font-thin"
                          value="data-viewer"
                          onClick={handleTabChange}>
                          <GridNineIcon />
                          <p className="text-sm font-thin select-none">
                            {t("Tabular Viewer")}
                          </p>
                        </TabsTrigger>
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
                      </TabsList>
                    </div>
                  </div>
                </Tabs>
                <DataTable
                  fileContent={selectedOutputData}
                  fileType={fileType}
                />
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
    </div>
  ) : null;
};

export default memo(DebugPanel);
