import { useState } from "react";

import {
  Button,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  Input,
  Label,
  Textarea,
} from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace, useDialogType } from "@flow/stores";

export const AddProject: React.FC = () => {
  const t = useT();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [, setDialogType] = useDialogType();
  const [showError, setShowError] = useState(false);
  const { createProject } = useProject();
  const [currentWorkspace] = useCurrentWorkspace();

  const handleClick = async () => {
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
    setDialogType(undefined);
  };

  return (
    <DialogContentWrapper>
      <DialogTitle>{t("New project")}</DialogTitle>
      <DialogContentWrapper>
        <DialogContentSection>
          <Label>{t("Project name")}</Label>
          <Input
            placeholder={t("Project name...")}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </DialogContentSection>
        <DialogContentSection>
          <Label>{t("Project description (optional)")}</Label>
          <Textarea
            placeholder={t("Project description...")}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </DialogContentSection>
        <div className="mt-2 flex flex-col gap-6">
          <div
            className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}
          >
            {t("Failed to create project")}
          </div>
        </div>
      </DialogContentWrapper>
      <DialogFooter>
        <Button
          className="self-end"
          disabled={!name || buttonDisabled}
          size="sm"
          onClick={handleClick}
        >
          {t("Create")}
        </Button>
      </DialogFooter>
    </DialogContentWrapper>
  );
};
