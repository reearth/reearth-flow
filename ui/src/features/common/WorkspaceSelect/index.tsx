import { useMemo } from "react";

import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Workspace } from "@flow/types";

type Props = {
  workspaces: Workspace[];
  selectedWorkspaceId?: string;
  onSelectWorkspace: (workspace: Workspace) => void;
};

const WorkspaceSelect: React.FC<Props> = ({
  workspaces,
  selectedWorkspaceId,
  onSelectWorkspace,
}) => {
  const t = useT();

  const personalWorkspaces = useMemo(
    () => workspaces.filter((w) => w.personal),
    [workspaces],
  );
  const teamWorkspaces = useMemo(
    () => workspaces.filter((w) => !w.personal),
    [workspaces],
  );

  if (workspaces.length === 0) {
    return (
      <p className="text-sm text-muted-foreground dark:font-extralight">
        {t("You don't have permission to create projects in any workspace.")}
      </p>
    );
  }

  return (
    <Select
      value={selectedWorkspaceId ?? null}
      onValueChange={(value) => {
        const workspace = workspaces.find((w) => w.id === value);
        if (workspace) onSelectWorkspace(workspace);
      }}
      items={workspaces.map((w) => ({ value: w.id, label: w.name }))}>
      <SelectTrigger className="w-full">
        <SelectValue placeholder={t("Select a workspace")} />
      </SelectTrigger>
      <SelectContent className="max-h-80 overflow-y-auto">
        {personalWorkspaces.length > 0 && (
          <SelectGroup>
            <SelectLabel className="text-xs text-muted-foreground">
              {t("Personal")}
            </SelectLabel>
            {personalWorkspaces.map((workspace) => (
              <SelectItem
                className="pl-3"
                key={workspace.id}
                value={workspace.id}>
                {workspace.name}
              </SelectItem>
            ))}
          </SelectGroup>
        )}
        {teamWorkspaces.length > 0 && (
          <SelectGroup>
            <SelectLabel className="text-xs text-muted-foreground">
              {t("Team Workspaces")}
            </SelectLabel>
            {teamWorkspaces.map((workspace) => (
              <SelectItem
                className="pl-3"
                key={workspace.id}
                value={workspace.id}>
                {workspace.name}
              </SelectItem>
            ))}
          </SelectGroup>
        )}
      </SelectContent>
    </Select>
  );
};

export { WorkspaceSelect };
