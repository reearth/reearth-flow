import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { Button, Input } from "@flow/components";
import { useCreateWorkspaceMutation } from "@flow/lib/gql";
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
  const createWorkspace = useCreateWorkspaceMutation();

  useEffect(() => {
    if (createWorkspace.isIdle || createWorkspace.isPending) return;

    if (createWorkspace.isError) {
      setShowError(true);
      setButtonDisabled(false);
    } else if (createWorkspace.isSuccess) {
      const workspaceId = createWorkspace.data?.createWorkspace?.workspace.id;
      setButtonDisabled(false);
      setShowError(false);
      setDialogType(undefined);
      navigate({ to: `/workspace/${workspaceId}` });
    }
  }, [createWorkspace, setShowError, setButtonDisabled, setDialogType, navigate]);

  const handleClick = () => {
    if (!name) return;
    createWorkspace.mutate({ input: { name } });
    setButtonDisabled(true);
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
