import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { Button, Input } from "@flow/components";
import {
  CreateWorkspaceMutation,
  useCreateWorkspaceMutation,
  useGetWorkspaceQuery,
} from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";

import { ContentHeader } from "../ContentHeader";
import { ContentSection } from "../ContentSection";

const AddWorkspace: React.FC = () => {
  const t = useT();
  const [name, setName] = useState<string | undefined>();
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [, setDialogType] = useDialogType();
  const navigate = useNavigate();
  const [showError, setShowError] = useState(false);
  const [createdWorkspaceId, setCreatedWorkspaceId] = useState<string | undefined>(undefined);
  const { data, ...rest } = useGetWorkspaceQuery();

  const workspaces = data?.me?.workspaces;

  useEffect(() => {
    if (!createdWorkspaceId || rest.isFetching) return;

    const createdWorkspace = workspaces?.find(w => w.id === createdWorkspaceId);
    if (createdWorkspace) {
      setButtonDisabled(false);
      setShowError(false);
      setDialogType(undefined);
      navigate({ to: `/workspace/${createdWorkspaceId}` });
    }
  }, [
    createdWorkspaceId,
    rest.isFetching,
    navigate,
    workspaces,
    setButtonDisabled,
    setShowError,
    setDialogType,
  ]);

  const handleSuccess = (data: CreateWorkspaceMutation) => {
    const workspaceId = data?.createWorkspace?.workspace?.id;
    setCreatedWorkspaceId(workspaceId);
  };

  const handleFailure = () => {
    setShowError(true);
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
