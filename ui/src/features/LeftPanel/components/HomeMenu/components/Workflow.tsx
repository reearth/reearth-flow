import { DropdownMenuItem } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogAtom } from "@flow/stores";

const WorkflowSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogAtom();
  return (
    <DropdownMenuItem onClick={() => setDialogType("workflow-settings")}>
      {t("Workflow")}
      {/* <DropdownMenuShortcut>⇧⌘F</DropdownMenuShortcut> */}
    </DropdownMenuItem>
  );
};

export { WorkflowSetting };
