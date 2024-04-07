import { DropdownMenuItem, DropdownMenuShortcut } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogAtom } from "@flow/stores";

const KeyboardSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogAtom();
  return (
    <DropdownMenuItem onClick={() => setDialogType("keyboard-settings")}>
      {t("Keyboard shortcuts")}
      <DropdownMenuShortcut>⇧⌘K</DropdownMenuShortcut>
    </DropdownMenuItem>
  );
};

export { KeyboardSetting };
