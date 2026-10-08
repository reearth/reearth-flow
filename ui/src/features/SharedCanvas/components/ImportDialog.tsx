import { memo, useCallback } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  Label,
} from "@flow/components";
import { WorkspaceSelect } from "@flow/features/common";
import { useT } from "@flow/lib/i18n";
import type { Workspace } from "@flow/types";

type Props = {
  workspaces: Workspace[];
  selectedWorkspace: Workspace | null;
  onSelectWorkspace: (workspace: Workspace) => void;
  onImportProject: () => void;
  onDialogClose: () => void;
};

const ImportDialog: React.FC<Props> = ({
  workspaces,
  selectedWorkspace,
  onSelectWorkspace,
  onImportProject,
  onDialogClose,
}) => {
  const t = useT();

  const handleSubmitImportProject = useCallback(() => {
    onImportProject();
    onDialogClose();
  }, [onImportProject, onDialogClose]);

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="sm">
        <DialogTitle>{t("Import Project")}</DialogTitle>
        <DialogContentWrapper>
          <Label>{t("Import Project to Workspace: ")}</Label>
          <DialogContentSection className="flex flex-row items-center">
            <WorkspaceSelect
              workspaces={workspaces}
              selectedWorkspaceId={selectedWorkspace?.id}
              onSelectWorkspace={onSelectWorkspace}
            />
          </DialogContentSection>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            disabled={!selectedWorkspace}
            onClick={handleSubmitImportProject}>
            {t("Import")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(ImportDialog);
