// import { useT } from "@flow/providers";

import { useState } from "react";

import { Button, Input } from "@flow/components";
import { useCreateWorkspaceMutation } from "@flow/lib/gql";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

import { ContentHeader } from "../ContentHeader";
import { ContentSection } from "../ContentSection";

const AddWorkspace: React.FC = () => {
  const t = useT();
  const [name, setName] = useState<string | undefined>();
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [, setDialogType] = useDialogType();

  const handleSuccess = () => {
    window.alert("Workspace created");
    setDialogType(undefined);
  };

  const handleFailure = () => {
    window.alert("Some error occurred. Please try again");
    setButtonDisabled(false);
  };

  const createWorkspace = useCreateWorkspaceMutation({
    onSuccess: handleSuccess,
    onError: handleFailure,
  });

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
