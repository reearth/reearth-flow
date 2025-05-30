import { ArrowSquareIn } from "@phosphor-icons/react";

import { Button, ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Workspace } from "@flow/types";

type Props = {
  workspaces: Workspace[];
  selectedWorkspace: Workspace | null;
  onSelectWorkspace: (workspace: Workspace) => void;
  onImportProject: () => void;
};

const ImportPopover: React.FC<Props> = ({
  workspaces,
  selectedWorkspace,
  onSelectWorkspace,
  onImportProject,
}) => {
  const t = useT();
  const [personalWorkspace, ...teamWorkspaces] =
    workspaces?.sort((a, b) => (a.personal ? -1 : b.personal ? 1 : 0)) || [];

  return (
    <div className="flex flex-col">
      <div className="flex flex-col gap-2 justify-between border-b p-4 pb-2">
        <h4 className="text-md dark:font-thin leading-none tracking-tight rounded-t-lg">
          {t("Import Project")}
        </h4>
        <p className="text-xs dark:font-light">
          {t("Select a workspace below to import the shared project into it.")}
        </p>
      </div>

      <div className="flex flex-col overflow-auto">
        <ScrollArea>
          <div className="flex flex-col">
            <p className="text-xs text-muted-foreground px-4 py-2">
              {t("Personal")}
            </p>
            <div
              onClick={() => onSelectWorkspace(personalWorkspace)}
              className={`flex cursor-pointer select-none justify-between gap-2 px-4 py-2 ${selectedWorkspace?.id === personalWorkspace.id ? "bg-card" : "hover:bg-card"}`}
              style={{ height: "100%" }}>
              <p className="flex-2 self-center text-sm dark:font-light">
                {personalWorkspace.name}
              </p>
            </div>
          </div>
          <div className="flex flex-col">
            <p className="text-xs text-muted-foreground px-4 py-2">
              {t("Team Workspaces")}
            </p>
            {teamWorkspaces.map((workspace) => (
              <div
                onClick={() => onSelectWorkspace(workspace)}
                className={`flex cursor-pointer select-none justify-between gap-2 px-4 py-2 ${selectedWorkspace?.id === workspace.id ? "bg-card" : "hover:bg-card"}`}
                style={{ height: "100%" }}>
                <p className="flex-2 self-center text-sm dark:font-light">
                  {workspace.name}
                </p>
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>

      <div className="flex justify-between gap-4 border-t p-2">
        <Button
          className="flex w-full gap-2"
          variant="outline"
          disabled={!selectedWorkspace}
          onClick={onImportProject}>
          <ArrowSquareIn weight="thin" size={18} />
          <p className="text-xs dark:font-light">{t("Import")}</p>
        </Button>
      </div>
    </div>
  );
};

export default ImportPopover;
