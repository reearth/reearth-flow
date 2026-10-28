import { cn } from "@flow/lib/utils";
import type { User } from "@flow/types";

import { Avatar, AvatarFallback, AvatarImage } from ".";

type Props = {
  user?: Pick<User, "name" | "photoURL">;
  className?: string;
};

const UserAvatar: React.FC<Props> = ({ user, className }) => (
  <Avatar className={cn("size-[18px]", className)}>
    {user?.photoURL && <AvatarImage src={user.photoURL} alt="" />}
    <AvatarFallback className="bg-zinc-600 text-[11px] font-normal text-white">
      {user?.name.trim().charAt(0).toUpperCase() || "?"}
    </AvatarFallback>
  </Avatar>
);

export { UserAvatar };
