import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { Button, Input, Label } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

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
    navigate({ to: "/workspace" });
  };

  const handleUpdateWorkspace = async () => {
    setLoading(true);
    setShowError(undefined);
    if (!currentWorkspace?.id || !workspaceName) return;
    const { workspace } = await updateWorkspace(currentWorkspace?.id, workspaceName);
    setLoading(false);
    if (!workspace) {
      setShowError("update");
      return;
    }
  };
  return (
    <div>
      <p className="text-lg font-extralight">{t("General Settings")}</p>
      <div className="flex flex-col gap-6 mt-4 max-w-[700px]">
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-name">{t("Workspace Name")}</Label>
          <Input
            id="workspace-name"
            placeholder={t("Workspace Name")}
            disabled={currentWorkspace?.personal || loading}
            value={workspaceName}
            onChange={e => setWorkspaceName(e.target.value)}
          />
        </div>
        <Button
          className="self-end"
          disabled={loading || !workspaceName || currentWorkspace?.personal}
          onClick={handleUpdateWorkspace}>
          {t("Save")}
        </Button>
        <Button
          variant={"destructive"}
          disabled={currentWorkspace?.personal || loading}
          className="self-end"
          onClick={() => handleDeleteWorkspace()}>
          {t("Delete Workspace")}
        </Button>
        <div className={`text-xs text-red-400 self-end ${showError ? "opacity-70" : "opacity-0"}`}>
          {showError === "delete" && t("Failed to delete Workspace")}
          {showError === "update" && t("Failed to update Workspace")}
        </div>
      </div>
    </div>
  );
};

export { GeneralSettings };
