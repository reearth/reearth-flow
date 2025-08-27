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

  // Extract users from awareness with stale client filtering
  useEffect(() => {
    if (!yDoc || !awareness) return;

    const STALE_THRESHOLD = 30000; // 30 seconds - same as cursor logic

    const handleAwarenessUpdate = () => {
      const states = awareness.getStates();
      const onlineUsers: User[] = [];
      const now = Date.now();

      states.forEach((state: any, clientId: number) => {
        // Skip self
        if (clientId === yDoc.clientID) return;

        if (state.user) {
          // Check if this client has a lastActive timestamp
          const lastActive = state.lastActive || now;
          const isStale = now - lastActive > STALE_THRESHOLD;

          // Only show users that are not stale
          if (!isStale) {
            onlineUsers.push({
              userId: clientId.toString(),
              userName: state.user.name || `User ${clientId}`,
            });
          } else {
            console.log(
              `Filtering out stale user ${clientId} (${state.user.name}) - inactive for ${now - lastActive}ms`,
            );
          }
        }
      });

      console.log(`Online users after filtering: ${onlineUsers.length}`);
      setUsers(onlineUsers);
    };

    awareness.on("update", handleAwarenessUpdate);
    handleAwarenessUpdate(); // Initial update

    // Periodic cleanup to re-evaluate stale users
    const cleanupInterval = setInterval(() => {
      handleAwarenessUpdate(); // Force re-evaluation of stale users
    }, 15000); // Every 15 seconds - same as cursor logic

    return () => {
      awareness.off("update", handleAwarenessUpdate);
      clearInterval(cleanupInterval);
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
            <div key={self.id} className="relative">
              <div className="flex h-6 w-6 items-center justify-center rounded-full bg-secondary ring-2 ring-background">
                <span className="text-xs font-medium">
                  {self.name.charAt(0).toUpperCase()}
                </span>
              </div>
            </div>
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
