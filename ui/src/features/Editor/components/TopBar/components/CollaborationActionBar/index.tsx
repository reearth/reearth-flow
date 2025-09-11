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
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
  showDialog: DialogOptions;
  spotlightUserClientId: number | null;
  onDialogOpen: (dialog: DialogOptions) => void;
  onDialogClose: () => void;
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
};

const CollaborationActionBar: React.FC<Props> = ({
  self,
  users,
  showDialog,
  spotlightUserClientId,
  onDialogOpen,
  onDialogClose,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
}) => {
  const t = useT();

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
          tooltipText={t("Collaborators")}
          tooltipOffset={tooltipOffset}
          onClick={() => onDialogOpen("collaboration")}>
          <div className="flex items-center -space-x-2">
            <div key={self?.clientId} className="relative">
              <div
                className="flex h-6 w-6 items-center justify-center rounded-full ring-2 ring-secondary/20"
                style={{ backgroundColor: self?.color || undefined }}>
                <span className="text-xs font-medium">
                  {self.userName.charAt(0).toUpperCase()}
                </span>
              </div>
            </div>
            {users &&
              Object.entries(users)
                .slice(0, 2)
                .map(([_key, value]) => {
                  return (
                    <div key={value.clientId} className="relative">
                      <div
                        className="flex h-6 w-6 items-center justify-center rounded-full ring-2 ring-secondary/20"
                        style={{ backgroundColor: value.color || undefined }}>
                        <span className="text-xs font-medium">
                          {value.userName.charAt(0).toUpperCase()}
                        </span>
                      </div>
                    </div>
                  );
                })}
            {users && Object.entries(users).length > 2 && (
              <div className="z-10 flex h-6 w-6 items-center justify-center rounded-full bg-secondary ring-2 ring-secondary/20">
                <span className="text-[10px] font-medium">
                  + {Object.entries(users).length - 2}
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
          <CollaborationPopover
            self={self}
            users={users}
            spotlightUserClientId={spotlightUserClientId}
            onSpotlightUserSelect={onSpotlightUserSelect}
            onSpotlightUserDeselect={onSpotlightUserDeselect}
          />
        )}
      </PopoverContent>
    </Popover>
  );
};

export default memo(CollaborationActionBar);
