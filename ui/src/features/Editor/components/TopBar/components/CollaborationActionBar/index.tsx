import { memo, useEffect, useState } from "react";
import * as Y from "yjs";

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

type User = {
  userId: string;
  userName: string;
  displayPictureUrl?: string;
  lastActive?: string;
};

type Props = {
  project?: Project;
  yDoc: Y.Doc | null;
  self?: any;
  awareness?: any;
  showDialog: DialogOptions;
  onDialogOpen: (dialog: DialogOptions) => void;
  onDialogClose: () => void;
};

const CollaborationActionBar: React.FC<Props> = ({
  yDoc,
  awareness,
  self,
  showDialog,
  onDialogOpen,
  onDialogClose,
}) => {
  const t = useT();
  const [users, setUsers] = useState<User[]>([]);

  // Extract users from awareness
  useEffect(() => {
    if (!yDoc || !awareness) return;

    const handleAwarenessUpdate = () => {
      const states = awareness.getStates();
      const onlineUsers: User[] = [];

      states.forEach((state: any, clientId: number) => {
        if (clientId === yDoc.clientID) return;

        if (state.user) {
          onlineUsers.push({
            userId: clientId.toString(),
            userName: state.user.name || `User ${clientId}`,
          });
        }
      });

      setUsers(onlineUsers);
    };

    awareness.on("update", handleAwarenessUpdate);
    handleAwarenessUpdate();

    return () => {
      awareness.off("update", handleAwarenessUpdate);
    };
  }, [yDoc, awareness]);

  const displayUsers = users.length > 0 ? users : [];

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
          <div className="flex items-center -space-x-3">
            {displayUsers.slice(0, 3).map((user) => {
              return user.displayPictureUrl ? (
                <div key={user.userId} className="relative">
                  <img
                    className="h-6 w-6 rounded-full ring-2 ring-background"
                    src={user.displayPictureUrl}
                    alt="User Avatar"
                  />
                </div>
              ) : (
                <div key={user.userId} className="relative">
                  <div className="flex h-6 w-6 items-center justify-center rounded-full bg-secondary ring-2 ring-background">
                    <span className="text-xs font-medium">
                      {user.userName.charAt(0).toUpperCase()}
                    </span>
                  </div>
                </div>
              );
            })}
            {users.length > 3 && (
              <div className="flex h-6 w-6 items-center justify-center rounded-full bg-muted ring-2 ring-background">
                <span className="text-xs font-medium">+{users.length - 3}</span>
              </div>
            )}
          </div>
        </ButtonWithTooltip>
      </PopoverTrigger>
      <PopoverContent
        sideOffset={16}
        className="w-60 bg-primary/50 backdrop-blur">
        {showDialog === "multiuser" && (
          <CollaborationPopover self={self} users={displayUsers} />
        )}
      </PopoverContent>
    </Popover>
  );
};

export default memo(CollaborationActionBar);
