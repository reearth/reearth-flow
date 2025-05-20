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
} from "@flow/components";
import VersionCanvas from "@flow/features/VersionCanvas";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";

type Props = {
  selectedProjectSnapshotVersion: number;
  versionPreviewYWorkflows: Y.Map<YWorkflow> | null;
  onDialogClose: () => void;
  onVersionConfirmationDialogOpen: () => void;
};

const VersionHistoryChangeDialog: React.FC<Props> = ({
  versionPreviewYWorkflows,
  onDialogClose,
  onVersionConfirmationDialogOpen,
}) => {
  const t = useT();
  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="xl">
        <DialogTitle>{t("Viewing Version")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-row items-center">
            <EditorComponent
              versionPreviewYWorkflows={versionPreviewYWorkflows}
            />
          </DialogContentSection>
          <div className="border-t border-primary" />
        </DialogContentWrapper>
        <DialogFooter>
          <Button onClick={onVersionConfirmationDialogOpen}>{"change"}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const EditorComponent: React.FC<{
  versionPreviewYWorkflows: Y.Map<YWorkflow> | null;
}> = ({ versionPreviewYWorkflows }) => {
  return !versionPreviewYWorkflows ? (
    <>ERRR</>
  ) : (
    <div className="h-[500px] w-full">
      <ReactFlowProvider>
        <VersionCanvas yWorkflows={versionPreviewYWorkflows} />
      </ReactFlowProvider>
    </div>
  );
};

export { VersionHistoryChangeDialog };
