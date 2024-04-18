import { DropdownMenuItem, DropdownMenuShortcut } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

const WorkspacesSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogType();
  return (
    <DropdownMenuItem onClick={() => setDialogType("workspaces-settings")}>
      {t("Workspaces")}
      <DropdownMenuShortcut>⇧⌘W</DropdownMenuShortcut>
    </DropdownMenuItem>
  );
};

export { WorkspacesSetting };
