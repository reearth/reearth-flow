import {
  CaretUp,
  CornersIn,
  CornersOut,
  Globe,
  GridNine,
  Minus,
  Warning,
} from "@phosphor-icons/react";
import { memo } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  LoadingSkeleton,
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

import { DataTable, GeoMap } from "./components";
import useHooks from "./hooks";

const DebugPreview: React.FC = () => {
  const t = useT();
  const {
    selectedDataURL,
    dataURLs,
    expanded,
    minimized,
    selectedOutputData,
    fileType,
    debugJobState,
    isLoadingData,
    showTempPossibleIssuesDialog,
    handleShowTempPossibleIssuesDialogClose,
    handleExpand,
    handleMinimize,
    handleTabChange,
    handleSelectedDataChange,
  } = useHooks();

  return debugJobState && dataURLs ? (
    <Tabs
      className={`pointer-events-auto w-[45vw] min-w-[700px] rounded-md p-1 bg-secondary transition-all ${minimized ? "h-[36px]" : expanded ? "h-[90vh] w-[90vw]" : "h-[500px]"}`}
      defaultValue="data-viewer">
      <div className="relative flex items-center p-1">
        <div className="absolute left-1 top-1">
          <Select
            defaultValue={dataURLs[0].key}
            value={selectedDataURL}
            onValueChange={handleSelectedDataChange}>
            <SelectTrigger className="h-[26px] max-w-[200px] border-none text-xs font-bold">
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
        <div className="flex w-full items-center justify-center gap-1 p-1">
          <TabsList className="gap-2">
            <TabsTrigger
              className="gap-1 bg-card font-thin"
              value="data-viewer"
              onClick={handleTabChange}>
              <GridNine />
              <p className="select-none text-sm font-thin">
                {t("Table Viewer")}
              </p>
            </TabsTrigger>
            <TabsTrigger
              className="gap-1 bg-card"
              value="3d-viewer"
              onClick={handleTabChange}>
              <Globe />
              <p className="select-none text-sm font-thin">{t("3D Viewer")}</p>
            </TabsTrigger>
          </TabsList>
        </div>
        <div className="absolute right-1 top-1 flex items-center gap-2">
          <div
            className="cursor-pointer rounded p-1 hover:bg-primary"
            onClick={handleMinimize}>
            {minimized ? <CaretUp weight="light" /> : <Minus weight="light" />}
          </div>
          {!minimized && (
            <div
              className="cursor-pointer rounded p-1 hover:bg-primary"
              onClick={handleExpand}>
              {expanded ? (
                <CornersIn weight="light" />
              ) : (
                <CornersOut weight="light" />
              )}
            </div>
          )}
        </div>
      </div>
      {isLoadingData ? (
        <LoadingSkeleton />
      ) : (
        <>
          <TabsContent
            className="h-[calc(100%-35px)] overflow-scroll"
            value="data-viewer">
            <DataTable fileContent={selectedOutputData} fileType={fileType} />
          </TabsContent>
          <TabsContent
            className="m-0 h-[calc(100%-32px)] p-1"
            value="3d-viewer">
            <GeoMap fileContent={selectedOutputData} fileType={fileType} />
          </TabsContent>
        </>
      )}
    </Tabs>
  ) : showTempPossibleIssuesDialog ? (
    <Dialog open={showTempPossibleIssuesDialog}>
      <DialogContent size="sm" hideCloseButton>
        <DialogHeader className="text-warning">
          <DialogTitle className="flex justify-center gap-1">
            <Warning weight="light" />
            {t("Warning")}
          </DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="text-sm font-light">
              {t("Your workflow completed without any output data.")}
            </p>
          </DialogContentSection>
          <DialogContentSection>
            <p className="text-sm font-light">
              {t(
                "Please review the logs to see if there were any errors during the workflow process.",
              )}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            size="sm"
            onClick={handleShowTempPossibleIssuesDialogClose}>
            {t("OK")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ) : null;
};

export default memo(DebugPreview);
