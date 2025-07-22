import { GlobeIcon, MapPinAreaIcon, WarningIcon } from "@phosphor-icons/react";
import { memo, useState } from "react";

import {
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
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

type Props = {
  fileType: SupportedDataTypes | null;
  selectedOutputData: any;
  debugJobState?: JobState;
  isLoadingData: boolean;
  dataURLs?: { key: string; name: string }[];
  showTempPossibleIssuesDialog: boolean;
  onShowTempPossibleIssuesDialogClose: () => void;
};
const DebugPreview: React.FC<Props> = ({
  fileType,
  debugJobState,
  selectedOutputData,
  dataURLs,
  isLoadingData,
  showTempPossibleIssuesDialog,
  onShowTempPossibleIssuesDialogClose,
}) => {
  const t = useT();
  const [enableClustering, setEnableClustering] = useState<boolean>(true);
  const [tabValue, setTabValue] = useState<string>("2d-viewer");

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
            <div className="flex items-center gap-2">
              <Checkbox
                checked={enableClustering}
                onCheckedChange={(checked) => setEnableClustering(!!checked)}
              />
              <span className="text-sm font-thin select-none">
                {t("Enable Clustering")}
              </span>
            </div>
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
              key={`2d-viewer-${enableClustering}`}
              fileContent={selectedOutputData}
              fileType={fileType}
              enableClustering={enableClustering}
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
