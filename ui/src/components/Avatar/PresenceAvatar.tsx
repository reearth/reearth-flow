import { cn } from "@flow/lib/utils";
import type { AwarenessUser } from "@flow/types";

import { Avatar, AvatarFallback, AvatarImage } from ".";

type Props = {
  user: Pick<AwarenessUser, "userName" | "color" | "photoURL">;
  size?: "sm" | "md";
};

// A collaborator in the editor: their photo, or the first two letters of
// their name on their cursor colour while it loads or when it cannot be shown.
const PresenceAvatar: React.FC<Props> = ({ user, size = "sm" }) => (
  <Avatar
    className={cn(
      "ring-2 ring-secondary/20 select-none",
      size === "sm" ? "size-6" : "size-8",
    )}>
    {user.photoURL && <AvatarImage src={user.photoURL} alt={user.userName} />}
    <AvatarFallback
      className={cn(
        "font-medium text-white",
        size === "sm" ? "text-xs" : "text-sm",
      )}
      style={{ backgroundColor: user.color || undefined }}>
      {user.userName.charAt(0).toUpperCase()}
      {user.userName.charAt(1)}
    </AvatarFallback>
  </Avatar>
);

export { PresenceAvatar };
