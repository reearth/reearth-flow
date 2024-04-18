import { DropdownMenuItem, DropdownMenuShortcut } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

const KeyboardSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogType();
  return (
    <DropdownMenuItem onClick={() => setDialogType("keyboard-instructions")}>
      {t("Keyboard shortcuts")}
      <DropdownMenuShortcut>⇧⌘K</DropdownMenuShortcut>
    </DropdownMenuItem>
  );
};

export { KeyboardSetting };
