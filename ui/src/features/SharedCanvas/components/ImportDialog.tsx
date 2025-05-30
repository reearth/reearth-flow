import { useCallback } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
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

  const [personalWorkspace, ...teamWorkspaces] =
    workspaces?.sort((a, b) => (a.personal ? -1 : b.personal ? 1 : 0)) || [];

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
            <Select
              value={selectedWorkspace?.id}
              onValueChange={(value) => {
                const workspace = workspaces.find((w) => w.id === value);
                if (workspace) onSelectWorkspace(workspace);
              }}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder={t("Select a workspace")} />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectLabel className="text-xs text-muted-foreground">
                    {t("Personal")}
                  </SelectLabel>
                  <SelectItem value={personalWorkspace.id}>
                    {personalWorkspace.name}
                  </SelectItem>
                </SelectGroup>
                <SelectGroup>
                  <SelectLabel className="text-xs text-muted-foreground">
                    {t("Team Workspaces")}
                  </SelectLabel>
                  {teamWorkspaces.map((workspace) => (
                    <SelectItem key={workspace.id} value={workspace.id}>
                      {workspace.name}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
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

export default ImportDialog;
