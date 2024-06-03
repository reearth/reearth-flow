import { Gear, Graph, Keyboard } from "@phosphor-icons/react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";

type Props = {
  className?: string;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownOffset?: number;
};

const ProjectSettings: React.FC<Props> = ({ className, dropdownPosition, dropdownOffset }) => {
  const t = useT();
  const [, setDialogType] = useDialogType();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className={className}>
        <Gear className="h-6 w-6" weight="thin" />
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="text-zinc-300 w-[200px]"
        side={dropdownPosition ?? "bottom"}
        align="end"
        sideOffset={dropdownOffset ?? 4}>
        {/* <DropdownMenuLabel>My Account</DropdownMenuLabel> */}
        <DropdownMenuItem className="gap-2" onClick={() => setDialogType("project-settings")}>
          <Graph />
          <p>{t("Project settings")}</p>
        </DropdownMenuItem>
        <DropdownMenuItem className="gap-2" onClick={() => setDialogType("keyboard-instructions")}>
          <Keyboard />
          <p>{t("Keyboard shortcuts")}</p>
        </DropdownMenuItem>
        {/* <DropdownMenuSeparator /> */}
        {/* <DropdownMenuItem onClick={handleLogout} className="gap-2">
        <SignOut className="w-[15px] h-[15px] stroke-1" />
        <p>{t("Log out")}</p>
      </DropdownMenuItem> */}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { ProjectSettings };
