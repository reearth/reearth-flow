import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { Button, DialogFooter, Input } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";

import { ContentHeader } from "../ContentHeader";
import { ContentSection } from "../ContentSection";

const AddWorkspace: React.FC = () => {
  const t = useT();
  const [, setDialogType] = useDialogType();
  const navigate = useNavigate();
  const { createWorkspace } = useWorkspace();

  const [name, setName] = useState("");
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [showError, setShowError] = useState(false);

  const handleClick = async () => {
    if (!name) return;
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
    setDialogType(undefined);

    navigate({ to: `/workspace/${workspace.id}` });
  };

  return (
    <>
      <ContentHeader title={t("Add Workspace")} />
      <ContentSection
        title=""
        content={
          <div className="flex flex-col gap-6 mt-2">
            <Input
              placeholder={t("Workspace name")}
              value={name}
              onChange={e => setName(e.target.value)}
            />
            <div className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
              {t("Failed to create workspace")}
            </div>
          </div>
        }
      />
      <DialogFooter>
        <Button
          className="self-end"
          disabled={!name || buttonDisabled}
          size="sm"
          onClick={handleClick}>
          {t("Create")}
        </Button>
      </DialogFooter>
    </>
  );
};

export { AddWorkspace };
