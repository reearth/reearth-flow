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
} from "@flow/components";
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
    latestProjectSnapshotVersion,
    previewDocRef,
    previewDocYWorkflows,
    selectedProjectSnapshotVersion,
    isFetching,
    isReverting,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    onRollbackProject,
    onPreviewVersion,
    onVersionSelection,
  } = useHooks({ projectId: project?.id ?? "", yDoc, onDialogClose });

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
            <VersionEditorComponent
              yDoc={yDoc}
              previewDocYWorkflows={previewDocYWorkflows}
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
          <Button
            disabled={!selectedProjectSnapshotVersion}
            onClick={() => setOpenVersionConfirmationDialog(true)}>
            {t("Revert")}
          </Button>
        </DialogFooter>
      </DialogContent>
      {isReverting && <LoadingSplashscreen />}
      {openVersionConfirmationDialog &&
        selectedProjectSnapshotVersion &&
        !isReverting && (
          <VersionConfirmationDialog
            selectedProjectSnapshotVersion={selectedProjectSnapshotVersion}
            onDialogClose={() => setOpenVersionConfirmationDialog(false)}
            onRollbackProject={onRollbackProject}
          />
        )}
    </Dialog>
  );
};

const VersionEditorComponent: React.FC<{
  yDoc: Y.Doc | null;
  previewDocYWorkflows: Y.Map<YWorkflow> | null;
}> = ({ previewDocYWorkflows, yDoc }) => {
  return (
    <div className="h-[500px] w-[575px]">
      {!previewDocYWorkflows && yDoc && (
        <ReactFlowProvider>
          <VersionCanvas yWorkflows={yDoc.getMap<YWorkflow>("workflows")} />
        </ReactFlowProvider>
      )}
      {previewDocYWorkflows && (
        <ReactFlowProvider>
          <VersionCanvas yWorkflows={previewDocYWorkflows} />
        </ReactFlowProvider>
      )}
    </div>
  );
};

export { VersionDialog };
