import { ReactFlowProvider } from "@xyflow/react";
import * as Y from "yjs";

import {
  Dialog,
  Button,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DialogFooter,
  LoadingSplashscreen,
  FlowLogo,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import VersionCanvas from "@flow/features/VersionCanvas";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import { Project } from "@flow/types";

import useHooks from "./hooks";
import { VersionConfirmationDialog } from "./VersionConfirmationDialog";
import { VersionHistoryList } from "./VersionHistoryList";

type Props = {
  project?: Project;
  yDoc: Y.Doc | null;
  onDialogClose: () => void;
};

const VersionDialog: React.FC<Props> = ({ project, yDoc, onDialogClose }) => {
  const t = useT();
  const {
    history,
    isFetching,
    isReverting,
    selectedProjectSnapshotVersion,
    latestProjectSnapshotVersion,
    versionPreviewYWorkflows,
    openVersionChangeDialog,
    setOpenVersionChangeDialog,
    onRollbackProject,
    onPreviewVersion,
    onVersionSelection,
    previewDocRef,
  } = useHooks({ projectId: project?.id ?? "", yDoc });

  const handleCloseDialog = () => {
    if (previewDocRef.current) {
      previewDocRef.current.destroy();
      previewDocRef.current = null;
    }
    onDialogClose();
  };

  return (
    <Dialog open={true} onOpenChange={handleCloseDialog}>
      <DialogContent size="2xl">
        <DialogTitle>{t("Viewing Version")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-row items-center">
            <EditorComponent
              versionPreviewYWorkflows={versionPreviewYWorkflows}
            />
            <VersionHistoryList
              latestProjectSnapshotVersion={latestProjectSnapshotVersion}
              history={history}
              selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
              onVersionSelection={onVersionSelection}
              isFetching={isFetching}
              onPreviewVersion={onPreviewVersion}
            />
          </DialogContentSection>
          <div className="border-t border-primary" />
        </DialogContentWrapper>
        <DialogFooter>
          <Button onClick={() => setOpenVersionChangeDialog(true)}>
            {t("Revert")}
          </Button>
        </DialogFooter>
      </DialogContent>
      {isReverting && <LoadingSplashscreen />}
      {openVersionChangeDialog &&
        selectedProjectSnapshotVersion &&
        !isReverting && (
          <VersionConfirmationDialog
            selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
            onDialogClose={() => setOpenVersionChangeDialog(false)}
            onRollbackProject={onRollbackProject}
          />
        )}
    </Dialog>
  );
};

const EditorComponent: React.FC<{
  versionPreviewYWorkflows: Y.Map<YWorkflow> | null;
}> = ({ versionPreviewYWorkflows }) => {
  const t = useT();

  return !versionPreviewYWorkflows ? (
    <BasicBoiler
      text={t("No Version Selected")}
      icon={<FlowLogo className="size-16 text-accent" />}
    />
  ) : (
    <div className="h-[500px] w-[575px]">
      <ReactFlowProvider>
        <VersionCanvas yWorkflows={versionPreviewYWorkflows} />
      </ReactFlowProvider>
    </div>
  );
};

export { VersionDialog };
