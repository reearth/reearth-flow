import {
  CaretUpIcon,
  CornersInIcon,
  CornersOutIcon,
  GlobeIcon,
  GridNineIcon,
  MapPinAreaIcon,
  MinusIcon,
  WarningIcon,
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

import { DataTable, ThreeDViewer, TwoDViewer } from "./components";
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
      className={`pointer-events-auto w-[45vw] min-w-[700px] rounded-md bg-secondary p-1 shadow-md shadow-secondary transition-all ${minimized ? "h-[36px]" : expanded ? "h-[90vh] w-[90vw]" : "h-[500px]"}`}
      defaultValue="data-viewer">
      <div className="relative flex items-center p-1">
        <div className="top-1 left-1">
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
        <div className="flex w-full items-center justify-center p-1">
          <TabsList className="gap-2">
            <TabsTrigger
              className="gap-1 bg-card font-thin"
              value="data-viewer"
              onClick={handleTabChange}>
              <GridNineIcon />
              <p className="text-sm font-thin select-none">
                {t("Table Viewer")}
              </p>
            </TabsTrigger>
            <TabsTrigger
              className="gap-1 bg-card"
              value="2d-viewer"
              onClick={handleTabChange}>
              <MapPinAreaIcon />
              <p className="text-sm font-thin select-none">{t("2D Viewer")}</p>
            </TabsTrigger>
            <TabsTrigger
              className="gap-1 bg-card"
              value="3d-viewer"
              onClick={handleTabChange}>
              <GlobeIcon />
              <p className="text-sm font-thin select-none">{t("3D Viewer")}</p>
            </TabsTrigger>
          </TabsList>
        </div>
        <div className="top-1 right-1 flex items-center gap-2">
          <div
            className="cursor-pointer rounded p-1 hover:bg-primary"
            onClick={handleMinimize}>
            {minimized ? (
              <CaretUpIcon weight="light" />
            ) : (
              <MinusIcon weight="light" />
            )}
          </div>
          {!minimized && (
            <div
              className="cursor-pointer rounded p-1 hover:bg-primary"
              onClick={handleExpand}>
              {expanded ? (
                <CornersInIcon weight="light" />
              ) : (
                <CornersOutIcon weight="light" />
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
            value="2d-viewer">
            <TwoDViewer fileContent={selectedOutputData} fileType={fileType} />
          </TabsContent>
          <TabsContent
            className="m-0 h-[calc(100%-32px)] p-1"
            value="3d-viewer">
            <ThreeDViewer
              fileContent={selectedOutputData}
              fileType={fileType}
            />
          </TabsContent>
        </>
      )}
    </Tabs>
  ) : showTempPossibleIssuesDialog ? (
    <Dialog open={showTempPossibleIssuesDialog}>
      <DialogContent size="sm" hideCloseButton>
        <DialogHeader className="text-warning">
          <DialogTitle className="flex justify-center gap-1">
            <WarningIcon weight="light" />
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
