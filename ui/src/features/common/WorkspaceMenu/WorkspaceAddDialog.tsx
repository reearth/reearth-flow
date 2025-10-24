import { useNavigate } from "@tanstack/react-router";
import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
} from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const WorkspaceAddDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const t = useT();
  const navigate = useNavigate();
  const { createWorkspace } = useWorkspace();
  const [currentWorkspace] = useCurrentWorkspace();

  const [name, setName] = useState("");
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [showError, setShowError] = useState(false);

  const handleClick = useCallback(async () => {
    if (!name || !currentWorkspace) return;
    setShowError(false);
    setButtonDisabled(true);
    const { workspace } = await createWorkspace(name);

    if (!workspace) {
      setShowError(true);
      setButtonDisabled(false);
      return;
    }

    setButtonDisabled(false);
    setShowError(false);
    onOpenChange(false);

    navigate({ to: `/workspaces/${workspace.id}/projects` });
  }, [name, currentWorkspace, navigate, createWorkspace, onOpenChange]);

  return (
    <Dialog open={isOpen} onOpenChange={(o) => onOpenChange(o)}>
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>{t("New workspace")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <Label>{t("Workspace name")}</Label>
            <Input
              placeholder={t("Workspace name...")}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </DialogContentSection>
          <div className="flex flex-col gap-6">
            <div
              className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
              {t("Failed to create workspace")}
            </div>
          </div>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            disabled={!name || buttonDisabled}
            size="sm"
            onClick={handleClick}>
            {t("Create")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { WorkspaceAddDialog };
