import { useState } from "react";

import { Button, Input } from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace, useDialogType } from "@flow/stores";

import { ContentHeader } from "../ContentHeader";
import { ContentSection } from "../ContentSection";

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
    <>
      <ContentHeader
        title={t("Add Project")}
        description={`Selected Workspace: ${currentWorkspace?.name}`}
      />
      <ContentSection
        title=""
        content={
          <div className="flex flex-col gap-6 mt-2">
            <Input
              placeholder={t("Project Name")}
              value={name}
              onChange={e => setName(e.target.value)}
            />
            <Input
              placeholder={t("Project Description")}
              value={description}
              onChange={e => setDescription(e.target.value)}
            />
            <div className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
              {t("Failed to create project")}
            </div>
            <Button
              disabled={!name || buttonDisabled}
              variant="outline"
              size="sm"
              onClick={handleClick}>
              {t("Create")}
            </Button>
          </div>
        }
      />
    </>
  );
};
