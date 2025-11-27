import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { Button, Input, Label } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import { WorkspaceDeletionDialog } from "./components";

type Errors = "delete" | "update";

const GeneralSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { deleteWorkspace, updateWorkspace } = useWorkspace();
  const navigate = useNavigate();
  const [showError, setShowError] = useState<Errors | undefined>(undefined);
  const [workspaceName, setWorkspaceName] = useState(currentWorkspace?.name);
  const [loading, setLoading] = useState(false);

  const handleDeleteWorkspace = async () => {
    setLoading(true);
    setShowError(undefined);
    if (!currentWorkspace) return;
    // TODO: this should trigger a pop up for confirming
    const { workspaceId } = await deleteWorkspace(currentWorkspace.id);

    if (!workspaceId) {
      setShowError("delete");
      return;
    }
    navigate({ to: "/" });
  };

  const handleUpdateWorkspace = async () => {
    setLoading(true);
    setShowError(undefined);
    if (!currentWorkspace?.id || !workspaceName) return;
    const { workspace } = await updateWorkspace(
      currentWorkspace?.id,
      workspaceName,
    );
    setLoading(false);
    if (!workspace) {
      setShowError("update");
      return;
    }
  };

  // currentWorkspace can be changed from the navigation
  useEffect(() => {
    setWorkspaceName(currentWorkspace?.name);
  }, [currentWorkspace]);

  return (
    <>
      <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
        <p className="text-lg font-light dark:font-extralight">
          {t("General Settings")}
        </p>
      </div>
      <div className="mt-4 flex max-w-[700px] flex-col gap-6">
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-name">{t("Workspace Name")}</Label>
          <Input
            id="workspace-name"
            placeholder={t("Workspace Name")}
            disabled={currentWorkspace?.personal || loading}
            value={workspaceName}
            onChange={(e) => setWorkspaceName(e.target.value)}
          />
        </div>
        <Button
          className="self-end"
          disabled={loading || !workspaceName || currentWorkspace?.personal}
          onClick={handleUpdateWorkspace}>
          {t("Save")}
        </Button>
        <WorkspaceDeletionDialog
          disabled={currentWorkspace?.personal || loading}
          onWorkspaceDelete={handleDeleteWorkspace}
        />
        <div
          className={`self-end text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
          {showError === "delete" && t("Failed to delete Workspace")}
          {showError === "update" && t("Failed to update Workspace")}
        </div>
      </div>
    </>
  );
};

export { GeneralSettings };
