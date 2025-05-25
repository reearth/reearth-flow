import { ReactFlowProvider } from "@xyflow/react";
import * as Y from "yjs";

import {
  Dialog,
  Button,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  LoadingSplashscreen,
  LoadingSkeleton,
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
        <DialogTitle>
          {t("Viewing Version: {{version}}", {
            version: selectedProjectSnapshotVersion
              ? selectedProjectSnapshotVersion
              : latestProjectSnapshotVersion?.version,
          })}
        </DialogTitle>
        <DialogContentWrapper className="p-0">
          <DialogContentSection className="flex flex-row items-center gap-0">
            <VersionEditorComponent
              yDoc={yDoc}
              previewDocYWorkflows={previewDocYWorkflows}
            />
            <div className="min-h-[532px] min-w-[327px] p-4 border-l overflow-y-auto place-self-start relative">
              <p className="text-md dark:font-thin pb-4">
                {t("Version History")}
              </p>
              {isFetching ? (
                <LoadingSkeleton className="max-h-[500px] min-w-[270px] place-self-start pt-40" />
              ) : (
                <VersionHistoryList
                  latestProjectSnapshotVersion={latestProjectSnapshotVersion}
                  history={history}
                  selectedProjectSnapshotVersion={
                    selectedProjectSnapshotVersion
                  }
                  onVersionSelection={onVersionSelection}
                  onPreviewVersion={onPreviewVersion}
                />
              )}
              <div className="flex border-t justify-end absolute w-full bg-secondary p-2 bottom-0 left-0">
                <Button
                  disabled={!selectedProjectSnapshotVersion}
                  onClick={() => setOpenVersionConfirmationDialog(true)}>
                  {t("Revert")}
                </Button>
              </div>
            </div>
          </DialogContentSection>
        </DialogContentWrapper>
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
    <div className="h-[570px] w-[575px]">
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
