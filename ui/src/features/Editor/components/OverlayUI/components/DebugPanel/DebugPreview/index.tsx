import { GlobeIcon, MapPinAreaIcon, WarningIcon } from "@phosphor-icons/react";
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
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { JobState } from "@flow/stores";
import type { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import { ThreeDViewer, TwoDViewer } from "./components";

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

  return debugJobState && dataURLs ? (
    <Tabs className="h-full w-full p-1" defaultValue="2d-viewer">
      <div className="p-1">
        <TabsList className="flex w-full justify-between p-1">
          <div className="flex gap-2">
            <TabsTrigger className="gap-1 bg-card" value="2d-viewer">
              <MapPinAreaIcon />
              <p className="text-sm font-thin select-none">{t("2D Viewer")}</p>
            </TabsTrigger>
            <TabsTrigger className="gap-1 bg-card" value="3d-viewer">
              <GlobeIcon />
              <p className="text-sm font-thin select-none">{t("3D Viewer")}</p>
            </TabsTrigger>
          </div>
        </TabsList>
      </div>
      {isLoadingData ? (
        <LoadingSkeleton />
      ) : (
        <>
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
            onClick={onShowTempPossibleIssuesDialogClose}>
            {t("OK")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ) : null;
};

export default memo(DebugPreview);
