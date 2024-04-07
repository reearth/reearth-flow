import { DropdownMenuItem, DropdownMenuShortcut } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogAtom } from "@flow/stores";

const WorkspacesSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogAtom();
  return (
    <DropdownMenuItem onClick={() => setDialogType("workspaces-settings")}>
      {t("Workspaces")}
      <DropdownMenuShortcut>⇧⌘W</DropdownMenuShortcut>
    </DropdownMenuItem>
  );
};

export { WorkspacesSetting };
