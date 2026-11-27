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
  isImporting: boolean;
  onSelectWorkspace: (workspace: Workspace) => void;
  onImportProject: () => Promise<boolean>;
  onDialogClose: () => void;
};

const ImportDialog: React.FC<Props> = ({
  workspaces,
  selectedWorkspace,
  isImporting,
  onSelectWorkspace,
  onImportProject,
  onDialogClose,
}) => {
  const t = useT();

  const handleSubmitImportProject = useCallback(async () => {
    const imported = await onImportProject();
    if (imported) onDialogClose();
  }, [onImportProject, onDialogClose]);

  const handleOpenChange = useCallback(() => {
    if (!isImporting) onDialogClose();
  }, [isImporting, onDialogClose]);

  return (
    <Dialog open={true} onOpenChange={handleOpenChange}>
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
            disabled={!selectedWorkspace || isImporting}
            onClick={handleSubmitImportProject}>
            {isImporting ? t("Importing...") : t("Import")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(ImportDialog);
