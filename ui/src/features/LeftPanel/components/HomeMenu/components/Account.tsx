import { DropdownMenuItem, DropdownMenuShortcut } from "@flow/components";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

const AccountSetting: React.FC = () => {
  const t = useT();
  const [_, setDialogType] = useDialogType();
  return (
    <DropdownMenuItem onClick={() => setDialogType("account-settings")}>
      {t("Account")}
      <DropdownMenuShortcut>⇧⌘P</DropdownMenuShortcut>
    </DropdownMenuItem>
  );
};

export { AccountSetting };
