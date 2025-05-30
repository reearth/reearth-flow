import { ArrowSquareIn, XCircle } from "@phosphor-icons/react";

import { Button, ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Workspace } from "@flow/types";

type Props = {
  workspaces: Workspace[];
  selectedWorkspaceId: string | null;
  onSelectWorkspace: (workspaceId: string) => void;
  onImportProject: () => void;
};

const ImportPopover: React.FC<Props> = ({
  workspaces,
  selectedWorkspaceId,
  onSelectWorkspace,
  onImportProject,
}) => {
  const t = useT();

  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-2 justify-between border-b p-4 pb-2">
        <h4 className="text-md self-center dark:font-thin leading-none tracking-tight rounded-t-lg">
          {t("Import Project into Workspace")}
        </h4>
      </div>
      <div className="flex flex-col overflow-auto">
        <ScrollArea>
          <div className="flex flex-col gap-2">
            {workspaces.map((workspace) => (
              <div
                onClick={() => onSelectWorkspace(workspace.id)}
                className={`flex cursor-pointer select-none justify-between gap-2 px-4 py-2 ${selectedWorkspaceId === workspace.id ? "bg-secondary" : "hover:bg-secondary"}`}
                style={{ height: "100%" }}>
                <p className="flex-2 self-center text-xs font-thin">
                  {workspace.name}
                </p>
                {workspace.personal && (
                  <div className="flex justify-end">
                    <p className="rounded border bg-logo/30 p-1 text-xs font-thin">
                      <span className="font-light">{"Personal"}</span>
                    </p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>

      <div className="flex justify-between gap-4 p-4 pt-0">
        <Button className="flex gap-2" variant="outline">
          <XCircle />
          <p className="text-xs dark:font-light">{t("Cancel")}</p>
        </Button>
        <Button
          className="flex gap-2"
          variant="outline"
          disabled={!selectedWorkspaceId}
          onClick={onImportProject}>
          <ArrowSquareIn weight="thin" size={18} />
          <p className="text-xs dark:font-light">{t("Import Project")}</p>
        </Button>
      </div>
    </div>
  );
};

export default ImportPopover;
