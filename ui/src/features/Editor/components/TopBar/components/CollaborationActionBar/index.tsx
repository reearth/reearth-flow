import { memo } from "react";

import {
  ButtonWithTooltip,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Project } from "@flow/types";

import type { DialogOptions } from "../../hooks";

import { CollaborationPopover } from "./components";

const tooltipOffset = 6;

type Props = {
  project?: Project;
  showDialog: DialogOptions;
  onDialogOpen: (dialog: DialogOptions) => void;
  onDialogClose: () => void;
};

const CollaborationActionBar: React.FC<Props> = ({
  showDialog,
  onDialogOpen,
  onDialogClose,
}) => {
  const t = useT();
  const mockUsers = [
    {
      userId: "1",
      userName: "Max Rebo",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1518791841217-8f162f1e1131?w=800&auto=format&fit=crop",
    },
    {
      userId: "2",
      userName: "Sy Snootles",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1517423440428-a5a00ad493e8?w=800&auto=format&fit=crop",
    },
    {
      userId: "3",
      userName: "Droopy McCool",
    },
    {
      userId: "4",
      userName: "Rystáll Sant",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1508672019048-805c876b67e2?w=800&auto=format&fit=crop",
      lastActive: "Active 8 hours ago",
    },
    {
      userId: "5",
      userName: "Greeata Jendowanian",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1543852786-1cf6624b9987?w=800&auto=format&fit=crop",
      lastActive: "Active 4 days ago",
    },
    {
      userId: "6",
      userName: "Lyn Me",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1548199973-03cce0bbc87b?w=800&auto=format&fit=crop",
      lastActive: "Active 12 days ago",
    },
    {
      userId: "7",
      userName: "Ak-rev",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1558788353-f76d92427f16?w=800&auto=format&fit=crop",
      lastActive: "Active 20 days ago",
    },
    {
      userId: "8",
      userName: "Umpass-stay",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1560807707-8cc77767d783?w=800&auto=format&fit=crop",
      lastActive: "Active 21 days ago",
    },
    {
      userId: "9",
      userName: "Doda Bodonawieedo",
      displayPictureUrl:
        "https://images.unsplash.com/photo-1504208434309-cb69f4fe52b0?w=800&auto=format&fit=crop",
      lastActive: "Active 30 days ago",
    },
  ];

  return (
    <Popover
      open={showDialog === "multiuser"}
      onOpenChange={(open) => {
        if (!open) onDialogClose();
      }}>
      <PopoverTrigger asChild>
        <ButtonWithTooltip
          className="p-1"
          variant={"ghost"}
          tooltipText={t("Collaboration")}
          tooltipOffset={tooltipOffset}
          onClick={() => onDialogOpen("multiuser")}>
          <div className="flex -space-x-3">
            {mockUsers.slice(0, 3).map((user) => {
              return user.displayPictureUrl ? (
                <img
                  key={user.userId}
                  className="h-6 w-6 rounded-full ring-background"
                  src={user.displayPictureUrl}
                  alt="User Avatar"
                />
              ) : (
                <div
                  key={user.userId}
                  className="flex h-6 w-6 items-center justify-center rounded-full bg-secondary ring-background">
                  <span className="text-xs font-medium">
                    {user.userName.charAt(0).toUpperCase()}
                  </span>
                </div>
              );
            })}
          </div>
        </ButtonWithTooltip>
      </PopoverTrigger>
      <PopoverContent
        sideOffset={16}
        className="w-60 bg-primary/50 backdrop-blur">
        {showDialog === "multiuser" && (
          <CollaborationPopover users={mockUsers} />
        )}
      </PopoverContent>
    </Popover>
  );
};

export default memo(CollaborationActionBar);
