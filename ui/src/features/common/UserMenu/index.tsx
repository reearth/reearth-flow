import { useState } from "react";

import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@flow/components";
import { useUser } from "@flow/lib/gql";

import { AccountMenuItems } from "./AccountMenuItems";
import { AccountUpdateDialog } from "./AccountUpdateDialog";

type Props = {
  className?: string;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownAlign?: "start" | "center" | "end";
  dropdownOffset?: number;
};

const UserMenu: React.FC<Props> = ({
  className,
  dropdownPosition = "right",
  dropdownAlign,
  dropdownOffset = 10,
}) => {
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  const [openAccountUpdateDialog, setOpenAccountUpdateDialog] = useState(false);

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger>
          <div className={`flex size-8 items-center gap-1 ${className}`}>
            <Avatar className="size-full">
              {me?.photoURL && <AvatarImage src={me.photoURL} alt={me.name} />}
              <AvatarFallback>
                {me?.name ? me.name.charAt(0).toUpperCase() : "?"}
              </AvatarFallback>
            </Avatar>
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="flex w-[220px] flex-col gap-0.5"
          side={dropdownPosition}
          align={dropdownAlign}
          sideOffset={dropdownOffset ?? 4}>
          <AccountMenuItems
            onAccountSettingsOpen={() => setOpenAccountUpdateDialog(true)}
          />
        </DropdownMenuContent>
      </DropdownMenu>
      {openAccountUpdateDialog && (
        <AccountUpdateDialog
          isOpen={openAccountUpdateDialog}
          onOpenChange={setOpenAccountUpdateDialog}
        />
      )}
    </>
  );
};

export { UserMenu };
