import {
  CaretDownIcon,
  CaretUpIcon,
  CodeIcon,
  CornersInIcon,
  CornersOutIcon,
  CubeIcon,
  MapTrifoldIcon,
  EyeIcon,
  MinusIcon,
  XIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useEffect, useRef, useState } from "react";

import {
  Button,
  IconButton,
  Label,
  LoadingSkeleton,
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import type { MapControls } from "@flow/components/visualizations/TilesViewer";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

import DebugLogs from "./DebugLogs";
import DebugPreview from "./DebugPreview";
import TableViewer from "./DebugPreview/components/TableViewer";
import FeatureDetails from "./DebugPreview/components/TableViewer/FeatureDetails";
import useHooks from "./hooks";
import ModelView from "./ModelView";
import OutputDataDownload from "./OutputDataDownload";
import TilesView from "./TilesView";
import { flyTargetOf } from "./TilesView/flyTarget";
import RowCard from "./TilesView/RowCard";
import UnloadedRowCard from "./TilesView/UnloadedRowCard";

const DebugPanel: React.FC = () => {
  const {
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
    selectedFeatureId,
    selectedFeature,
    legacyPreview,
    setLegacyPreview,
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
    loadedRowsHave2D,
    detailsOverlayOpen,
    detailsFeature,
    formattedData,
    handleFeatureSelect,
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
    // Data properties
    detectedGeometryType,
    visualizerType,
    totalFeatures,
    isLoadingData,
  } = useHooks();
  const t = useT();
  const [tabValue, setTabValue] = useState("debug-logs");

  // With the map open, the selected row shows as a card over it; its full
  // details take the map's place until the user goes back to it.
  const [rowOverMap, setRowOverMap] = useState(false);
  const [mapControls, setMapControls] = useState<MapControls>();

  // Flies the map to a row, where its coordinates place it on the globe.
  const flyToRow = useCallback(
    (row: any) => {
      const target = flyTargetOf(row);
      if (target) mapControls?.flyTo(target);
    },
    [mapControls],
  );
  const showingRowOverMap = rowOverMap && !!selectedFeature;
  // Once nothing is selected, the next row starts on the map again.
  const [hadSelection, setHadSelection] = useState(!!selectedFeature);
  if (hadSelection !== !!selectedFeature) {
    setHadSelection(!!selectedFeature);
    if (!selectedFeature) setRowOverMap(false);
  }

  // The selected row, as its details or as a 3D model.
  const renderRowDetails = (onBack?: () => void) =>
    modelViewRequest ? (
      <ModelView
        request={modelViewRequest}
        openError={modelViewOpenError}
        onRetry={handleOpenIn3D}
        onBack={handleCloseModelView}
      />
    ) : (
      <FeatureDetails
        variant="pane"
        feature={selectedFeature}
        onClose={onBack}
        closeLabel={t("Back to map")}
        actions={
          canOpenIn3D && (
            <Button
              variant="outline"
              size="sm"
              className="h-7 gap-1 text-xs"
              onClick={handleOpenIn3D}>
              <CubeIcon size={14} />
              {t("Open in 3D")}
            </Button>
          )
        }
      />
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
      className={`absolute bottom-2 z-30 ${fullscreenDebug ? "" : "left-1/2 -translate-x-1/2"}`}>
      <div
        className={`${fullscreenDebug ? "fixed inset-0" : ""} flex items-end`}>
        <Tabs
          className={`pointer-events-auto z-30 border border-border bg-secondary/70 p-1 shadow-md shadow-[black]/10 backdrop-blur  transition-all dark:border-primary dark:shadow-secondary ${minimized ? "h-[42px] w-[92vw] rounded-xl" : fullscreenDebug ? "h-[100vh] w-[100vw] rounded-none" : expanded ? "h-[65vh] w-[99vw] rounded-xl" : "h-[45vh] w-[92vw] rounded-xl"}`}
          value={tabValue}
          defaultValue="debug-logs"
          onValueChange={setTabValue}>
          <div
            className="relative flex justify-between"
            onDoubleClick={handleExpand}>
            <div className="flex w-fit items-center">
              <TabsList className="gap-2">
                <TabsTrigger
                  className="group h-8 gap-1 border border-transparent bg-card font-light data-active:border-logo/40 dark:font-thin"
                  value="debug-logs"
                  onClick={handleTabChange}>
                  <CodeIcon className="group-data-active:fill-logo" />
                  <p className="text-sm select-none">{t("Workflow Logs")}</p>
                </TabsTrigger>
                <TabsTrigger
                  className="group h-8 gap-1 border border-transparent bg-card font-light data-active:border-logo/40 dark:font-thin"
                  value="debug-viewer"
                  disabled={!dataURLs?.length}
                  onClick={handleTabChange}>
                  <EyeIcon className="group-data-active:fill-logo" />
                  <p className="text-sm select-none">{t("Data Preview")}</p>
                </TabsTrigger>
              </TabsList>
              <div className="ml-2 h-full w-1 border-l" />
              <OutputDataDownload
                outputData={outputDataForDownload}
                archiveName={`job_${debugJobId}`}
              />
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
            keepMounted={
              debugJobIdRef.current !== debugJobId ? undefined : true
            }
            hidden={tabValue !== "debug-logs"}>
            <DebugLogs
              debugJobId={debugJobId}
              isJobActive={isDebugJobActive}
              onExitFullscreen={handleFullscreenExit}
            />
          </TabsContent>
          {dataURLs && (
            <TabsContent
              value="debug-viewer"
              keepMounted={true}
              hidden={tabValue !== "debug-viewer"}
              className="h-[calc(100%-32px)] overflow-hidden">
              <ResizablePanelGroup orientation="horizontal">
                {/* Sizes are strings because a bare number means pixels. Each
                    panel has an id because the ones beside the table come and
                    go. */}
                <ResizablePanel
                  id="debug-table"
                  defaultSize="60%"
                  minSize="20%"
                  className="flex flex-col">
                  <div className="flex items-center justify-between gap-2 py-2">
                    <Select
                      defaultValue={dataURLs[0].key}
                      value={selectedDataURL}
                      onValueChange={(v) =>
                        v != null && handleSelectedDataChange(v)
                      }
                      items={dataURLs.map(({ key, name }) => ({
                        value: key,
                        label: name,
                      }))}>
                      <SelectTrigger className="h-[26px] w-auto max-w-[300px] text-xs font-bold">
                        <SelectValue
                          placeholder={t("Select Data to Preview")}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {dataURLs.map(({ key, name }) => (
                          <SelectItem
                            key={key}
                            value={key}
                            className="group relative z-0 pr-8">
                            {name}
                            <IconButton
                              icon={<XIcon weight="light" />}
                              variant={"default"}
                              className="absolute top-1/2 right-2 z-50 h-4 w-4 -translate-y-1/2 bg-accent opacity-0 transition-opacity group-hover:opacity-100 hover:text-destructive"
                              onPointerDown={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                handleRemoveDataURL(key);
                              }}
                            />
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <div className="flex items-center gap-2 pr-1">
                      {!legacyPreview && (
                        <Button
                          variant="outline"
                          size="sm"
                          className="h-[26px] gap-1 text-xs"
                          disabled={!canShowOnMap}
                          onClick={handleShowOnMap}>
                          <MapTrifoldIcon size={14} />
                          {t("Show on map")}
                        </Button>
                      )}
                      <Label
                        htmlFor="debug-legacy-preview"
                        className="text-xs font-light text-muted-foreground">
                        {t("Legacy preview")}
                      </Label>
                      <Switch
                        id="debug-legacy-preview"
                        checked={legacyPreview}
                        onCheckedChange={setLegacyPreview}
                      />
                    </div>
                  </div>
                  <div className="min-h-0 flex-1">
                    <TableViewer
                      fileContent={selectedOutputData}
                      selectedFeatureId={selectedFeatureId}
                      onSingleClick={handleRowSingleClick}
                      onDoubleClick={(row: any) => {
                        handleRowDoubleClick(row);
                        if (mapViewRequest) flyToRow(row);
                      }}
                      detectedGeometryType={detectedGeometryType || undefined}
                      totalFeatures={totalFeatures || undefined}
                      detailsOverlayOpen={detailsOverlayOpen}
                      detailsFeature={detailsFeature}
                      formattedData={formattedData}
                      onShowFeatureDetailsOverlay={
                        handleShowFeatureDetailsOverlay
                      }
                    />
                  </div>
                </ResizablePanel>
                {!legacyPreview && (mapViewRequest || selectedFeature) && (
                  <>
                    {!minimized && (
                      <ResizableHandle className="mx-2 w-1" withHandle />
                    )}
                    <ResizablePanel
                      id="debug-details"
                      defaultSize="40%"
                      minSize="20%">
                      {mapViewRequest ? (
                        <>
                          {/* Hidden rather than removed while the row's
                              details show, so going back is instant. */}
                          <div
                            className={cn(
                              "h-full",
                              showingRowOverMap && "hidden",
                            )}>
                            <TilesView
                              request={mapViewRequest}
                              openError={mapViewOpenError}
                              selectedRow={selectedFeature?._row}
                              overlay={({ view }) =>
                                selectedFeature ? (
                                  <RowCard
                                    feature={selectedFeature}
                                    format={view.format}
                                    onZoomTo={
                                      mapControls &&
                                      flyTargetOf(selectedFeature)
                                        ? () => flyToRow(selectedFeature)
                                        : undefined
                                    }
                                    onShowDetails={() => setRowOverMap(true)}
                                    onOpenIn3D={
                                      canOpenIn3D
                                        ? () => {
                                            setRowOverMap(true);
                                            handleOpenIn3D();
                                          }
                                        : undefined
                                    }
                                  />
                                ) : (
                                  pickedUnloadedRow && (
                                    <UnloadedRowCard
                                      loadedRows={loadedRowCount}
                                    />
                                  )
                                )
                              }
                              onPickRow={handlePickRow}
                              onRetry={handleShowOnMap}
                              onControls={setMapControls}
                              hasLoaded2D={loadedRowsHave2D}
                              onClose={() => {
                                setRowOverMap(false);
                                handleCloseMap();
                              }}
                            />
                          </div>
                          {showingRowOverMap &&
                            renderRowDetails(() => setRowOverMap(false))}
                        </>
                      ) : (
                        renderRowDetails()
                      )}
                    </ResizablePanel>
                  </>
                )}
                {legacyPreview && visualizerType && (
                  <>
                    {!minimized && (
                      <ResizableHandle className="mx-2 w-1" withHandle />
                    )}
                    <ResizablePanel
                      id="debug-legacy-viewer"
                      defaultSize="40%"
                      minSize="20%">
                      {isLoadingData ? (
                        <div className="flex h-full items-center justify-center">
                          <div className="text-center text-muted-foreground">
                            <LoadingSkeleton className="mb-4" />
                            <p className="text-sm">{t("Loading data...")}</p>
                          </div>
                        </div>
                      ) : (
                        <DebugPreview
                          debugJobState={debugJobState}
                          dataURLs={dataURLs}
                          selectedOutputData={selectedOutputData}
                          selectedFeatureId={selectedFeatureId}
                          cesiumViewerRef={cesiumViewerRef}
                          onSelectedFeature={handleFeatureSelect}
                          onFlyToSelectedFeature={handleFlyToSelectedFeature}
                          onShowFeatureDetailsOverlay={
                            handleShowFeatureDetailsOverlay
                          }
                          detailsOverlayOpen={detailsOverlayOpen}
                          // Data detection props
                          detectedGeometryType={detectedGeometryType}
                          visualizerType={visualizerType}
                        />
                      )}
                    </ResizablePanel>
                  </>
                )}
              </ResizablePanelGroup>
            </TabsContent>
          )}
        </Tabs>
      </div>
    </div>
  ) : null;
};

export default memo(DebugPanel);
