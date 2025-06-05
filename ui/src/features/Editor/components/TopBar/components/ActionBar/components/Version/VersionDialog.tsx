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
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Project } from "@flow/types";

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
    isLoadingPreview,
    isReverting,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    onRollbackProject,
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
      <DialogContent className="w-[90vw] h-[90vh] max-w-none max-h-none">
        <DialogTitle>
          {t("Viewing Version: {{version}}", {
            version: selectedProjectSnapshotVersion
              ? selectedProjectSnapshotVersion
              : latestProjectSnapshotVersion?.version,
          })}
        </DialogTitle>
        <DialogContentWrapper className="p-0 h-full">
          <DialogContentSection className="flex flex-row gap-0 h-full overflow-hidden">
            {isLoadingPreview ? (
              <LoadingSkeleton />
            ) : (
              <div className="flex-1 overflow-auto">
                <VersionEditorComponent
                  yDoc={yDoc}
                  previewDocYWorkflows={previewDocYWorkflows}
                />
              </div>
            )}
            <div className="relative w-[30vw] min-w-[320px] max-w-[500px] h-full border-l flex flex-col">
              <p className="text-md dark:font-thin pl-4 pt-4">
                {t("Version History")}
              </p>
              <div className="flex-1 overflow-y-auto p-4 pb-[70px]">
                {isFetching ? (
                  <LoadingSkeleton />
                ) : (
                  <VersionHistoryList
                    latestProjectSnapshotVersion={latestProjectSnapshotVersion}
                    history={history}
                    selectedProjectSnapshotVersion={
                      selectedProjectSnapshotVersion
                    }
                    onVersionSelection={onVersionSelection}
                  />
                )}
              </div>
              <div className="absolute bottom-17 left-0 w-full bg-secondary border-t p-2 flex justify-end">
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
}> = ({ yDoc, previewDocYWorkflows }) => {
  return (
    <div className="w-full h-full overflow-hidden">
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
