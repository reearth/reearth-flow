import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser, User } from "@flow/types";

import CollaborationCard from "./CollaborationCard";

type Props = {
  self?: User;
  users?: Record<string, AwarenessUser>;
};

const CollaborationPopover: React.FC<Props> = ({ self, users }) => {
  const t = useT();
  if (users) {
    {
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
      });
    }
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="border-b p-2">
        <div className="flex items-center gap-2 ">
          <div
            key={self?.id}
            className="flex h-10 w-10 items-center justify-center rounded-full bg-secondary ring-background">
            <span className="text-sm font-medium">
              {self?.name.charAt(0).toUpperCase()}
            </span>
          </div>
          <div className="flex flex-col items-start">
            <span className="text-sm dark:font-light">{self?.name}</span>
          </div>
        </div>
      </div>
      <ScrollArea>
        <div className="flex max-h-[250px] flex-col gap-2">
          <div className="flex flex-col gap-2 p-2 pt-0 pb-2">
            <span className="text-sm opacity-55 dark:font-light">
              {t("Currently Viewing")}
            </span>
            {users &&
              Object.entries(users).map(([_key, value]) => {
                return <CollaborationCard userName={value.userName} />;
              })}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
};

export default CollaborationPopover;
