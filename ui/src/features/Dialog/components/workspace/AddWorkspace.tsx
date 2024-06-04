import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { Button, Input } from "@flow/components";
import { useCreateWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";

import { ContentHeader } from "../ContentHeader";
import { ContentSection } from "../ContentSection";

const AddWorkspace: React.FC = () => {
  const t = useT();
  const [name, setName] = useState("");
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [, setDialogType] = useDialogType();
  const navigate = useNavigate();
  const [showError, setShowError] = useState(false);
  const { createWorkspace } = useCreateWorkspace();

  const handleClick = async () => {
    if (!name) return;
    setShowError(false);
    setButtonDisabled(true);
    try {
      const workspace = await createWorkspace(name);

      if (!workspace) {
        throw new Error("Workspace not created properly");
      }

      setButtonDisabled(false);
      setShowError(false);
      setDialogType(undefined);

      navigate({ to: `/workspace/${workspace.id}` });
    } catch (err) {
      setShowError(true);
      setButtonDisabled(false);
    }
  };

  return (
    <>
      <ContentHeader title={t("Add Workspace")} />
      <ContentSection
        title=""
        content={
          <div className="flex flex-col gap-6 mt-2">
            <Input
              placeholder={t("Add workspace name")}
              value={name}
              onChange={e => setName(e.target.value)}
            />
            <div className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
              {t("Failed to create workspace")}
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

export { AddWorkspace };
