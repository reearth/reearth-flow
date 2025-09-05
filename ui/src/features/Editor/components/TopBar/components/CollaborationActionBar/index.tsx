import { memo } from "react";

import {
  ButtonWithTooltip,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser } from "@flow/types";

import type { DialogOptions } from "../../hooks";

import { CollaborationPopover } from "./components";

const tooltipOffset = 6;

type Props = {
  users?: Record<string, AwarenessUser>;
  showDialog: DialogOptions;
  onDialogOpen: (dialog: DialogOptions) => void;
  onDialogClose: () => void;
};

const CollaborationActionBar: React.FC<Props> = ({
  users,
  showDialog,
  onDialogOpen,
  onDialogClose,
}) => {
  const t = useT();
  console.log("users", users);

  return (
    <Popover
      open={showDialog === "collaboration"}
      onOpenChange={(open) => {
        if (!open) onDialogClose();
      }}>
      <PopoverTrigger asChild>
        <ButtonWithTooltip
          className="p-1"
          variant={"ghost"}
          tooltipText={t("Collaboration")}
          tooltipOffset={tooltipOffset}
          onClick={() => onDialogOpen("collaboration")}>
          <div className="flex items-center -space-x-3">
            {users &&
              Object.entries(users).map(([_key, value]) => {
                return (
                  <div key={value.userName} className="relative">
                    <div className="flex h-6 w-6 items-center justify-center rounded-full bg-secondary ring-2 ring-background">
                      <span className="text-xs font-medium">
                        {value.userName?.charAt(0).toUpperCase()}
                      </span>
                    </div>
                  </div>
                );
              })}
            {users && Object.entries(users).length > 3 && (
              <div className="flex h-6 w-6 items-center justify-center rounded-full bg-muted ring-2 ring-background">
                <span className="text-xs font-medium">
                  +{Object.entries(users).length - 3}
                </span>
              </div>
            )}
          </div>
        </ButtonWithTooltip>
      </PopoverTrigger>
      <PopoverContent
        sideOffset={16}
        className="w-60 bg-primary/50 backdrop-blur">
        {showDialog === "collaboration" && (
          <CollaborationPopover users={users} />
        )}
      </PopoverContent>
    </Popover>
  );
};

export default memo(CollaborationActionBar);
