import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { Button, Input, Label } from "@flow/components";
import { useDeleteWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

const GeneralSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { deleteWorkspace } = useDeleteWorkspace();
  const navigate = useNavigate();
  const [showError, setShowError] = useState(false);

  const handleDeleteWorkspace = async () => {
    setShowError(false);
    // TODO: this trigger a pop up for confirming
    try {
      if (!currentWorkspace) return;
      await deleteWorkspace(currentWorkspace.id);
      navigate({ to: "/workspace" });
    } catch (err) {
      setShowError(true);
      console.log(err);
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
            defaultValue={currentWorkspace?.name}
            readOnly={true}
          />
        </div>
        {/* <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-description">{t("Workspace Description")}</Label>
          <Input
            id="workspace-description"
            placeholder={t("Workspace Description")}
            defaultValue={currentWorkspace?.description}
          />
        </div> */}
        <Button className="self-end">{t("Save")}</Button>
        <Button
          variant={"destructive"}
          disabled={currentWorkspace?.personal}
          className="self-end"
          onClick={() => handleDeleteWorkspace()}>
          {t("Delete Workspace")}
        </Button>
        <div className={`text-xs text-red-400 self-end ${showError ? "opacity-70" : "opacity-0"}`}>
          {t("Failed to delete workspace")}
        </div>
      </div>
    </div>
  );
};

export { GeneralSettings };
