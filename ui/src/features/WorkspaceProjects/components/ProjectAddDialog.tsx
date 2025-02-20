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
  TextArea,
} from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const ProjectAddDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const t = useT();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [showError, setShowError] = useState(false);
  const { createProject } = useProject();
  const [currentWorkspace] = useCurrentWorkspace();

  const handleClick = useCallback(async () => {
    if (!name || !currentWorkspace) return;
    setShowError(false);
    setButtonDisabled(true);
    const { project } = await createProject({
      name,
      description,
      workspaceId: currentWorkspace.id,
    });

    if (!project) {
      setShowError(true);
      setButtonDisabled(false);
      return;
    }

    setButtonDisabled(false);
    setShowError(false);
    onOpenChange(false);
  }, [name, description, currentWorkspace, createProject, onOpenChange]);

  return (
    <Dialog open={isOpen} onOpenChange={(o) => onOpenChange(o)}>
      <DialogContent size="md" position="off-center">
        <DialogHeader>
          <DialogTitle>{t("New Project")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <Label>{t("Project Name")}</Label>
            <Input
              placeholder={t("Project name...")}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </DialogContentSection>
          <DialogContentSection>
            <Label>{t("Project Description (optional)")}</Label>
            <TextArea
              placeholder={t("Project description...")}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </DialogContentSection>
          <div className="mt-2 flex flex-col gap-6">
            <div
              className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
              {t("Failed to create project")}
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

export { ProjectAddDialog };
