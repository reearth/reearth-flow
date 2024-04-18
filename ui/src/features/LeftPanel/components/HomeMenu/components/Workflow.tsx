import { DropdownMenuItem } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

const WorkflowSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogType();
  return (
    <DropdownMenuItem onClick={() => setDialogType("workflow-settings")}>
      {t("Workflow")}
      {/* <DropdownMenuShortcut>⇧⌘F</DropdownMenuShortcut> */}
    </DropdownMenuItem>
  );
};

export { WorkflowSetting };
