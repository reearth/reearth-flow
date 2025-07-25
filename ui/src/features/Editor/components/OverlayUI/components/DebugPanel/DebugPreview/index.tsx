import {
  DotsThreeVerticalIcon,
  GlobeIcon,
  MapPinAreaIcon,
  TargetIcon,
  WarningIcon,
} from "@phosphor-icons/react";
import { memo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  LoadingSkeleton,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { JobState } from "@flow/stores";
import type { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import ThreeDViewer from "./components/ThreeDViewer";
import TwoDViewer from "./components/TwoDViewer";
import useHooks from "./hooks";

type Props = {
  fileType: SupportedDataTypes | null;
  selectedOutputData: any;
  debugJobState?: JobState;
  isLoadingData: boolean;
  dataURLs?: { key: string; name: string }[];
  showTempPossibleIssuesDialog: boolean;
  selectedFeature: any;
  convertedSelectedFeature: any;
  enableClustering?: boolean;
  mapRef: React.RefObject<maplibregl.Map | null>;
  onShowTempPossibleIssuesDialogClose: () => void;
  onSelectedFeature: (value: any) => void;
  onEnableClusteringChange: (value: boolean) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
};
const DebugPreview: React.FC<Props> = ({
  fileType,
  debugJobState,
  selectedOutputData,
  dataURLs,
  isLoadingData,
  showTempPossibleIssuesDialog,
  enableClustering,
  mapRef,
  selectedFeature,
  convertedSelectedFeature,
  onShowTempPossibleIssuesDialogClose,
  onSelectedFeature,
  onEnableClusteringChange,
  onFlyToSelectedFeature,
}) => {
  const t = useT();
  const [tabValue, setTabValue] = useState<string>("2d-viewer");

  const { handleMapLoad } = useHooks({
    mapRef,
    selectedOutputData,
  });

  return debugJobState && dataURLs ? (
    <Tabs
      className="h-full w-full p-1"
      defaultValue={tabValue}
      onValueChange={setTabValue}>
      <div className="p-1">
        <TabsList className="flex w-full justify-between p-1">
          <div className="flex gap-2">
            <TabsTrigger
              className="gap-1 bg-card"
              value="2d-viewer"
              onClick={() => setTabValue("2d-viewer")}>
              <MapPinAreaIcon />
              <p className="text-sm font-thin select-none">{t("2D Viewer")}</p>
            </TabsTrigger>
            <TabsTrigger
              className="gap-1 bg-card"
              value="3d-viewer"
              onClick={() => setTabValue("3d-viewer")}>
              <GlobeIcon />
              <p className="text-sm font-thin select-none">{t("3D Viewer")}</p>
            </TabsTrigger>
          </div>
          {tabValue === "2d-viewer" && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <IconButton
                  className="w-[25px]"
                  tooltipText={t("Additional actions")}
                  tooltipOffset={12}
                  icon={<DotsThreeVerticalIcon size={18} />}
                />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuCheckboxItem
                  checked={enableClustering}
                  onCheckedChange={(checked) =>
                    onEnableClusteringChange(!!checked)
                  }>
                  {t("Enable Clustering")}
                </DropdownMenuCheckboxItem>
                <DropdownMenuItem onClick={() => handleMapLoad(true)}>
                  <TargetIcon />
                  {t("Center Data")}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </TabsList>
      </div>
      {isLoadingData ? (
        <LoadingSkeleton />
      ) : (
        <>
          <TabsContent
            className="m-0 h-[calc(100%-32px)] p-1"
            value="2d-viewer">
            <TwoDViewer
              fileContent={selectedOutputData}
              fileType={fileType}
              enableClustering={enableClustering}
              selectedFeature={selectedFeature}
              convertedSelectedFeature={convertedSelectedFeature}
              mapRef={mapRef}
              onMapLoad={handleMapLoad}
              onSelectedFeature={onSelectedFeature}
              onFlyToSelectedFeature={onFlyToSelectedFeature}
            />
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
            onClick={onShowTempPossibleIssuesDialogClose}>
            {t("OK")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ) : null;
};

export default memo(DebugPreview);
