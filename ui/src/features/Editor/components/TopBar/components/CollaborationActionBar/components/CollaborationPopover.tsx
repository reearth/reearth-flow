import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser } from "@flow/types";

import CollaborationCard from "./CollaborationCard";

type Props = {
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
};

const CollaborationPopover: React.FC<Props> = ({ self, users }) => {
  const t = useT();

  return (
    <div className="flex flex-col gap-2">
      <div className="p-2">
        <CollaborationCard userName={self?.userName} />
      </div>
      {users && Object.entries(users).length >= 1 && (
        <ScrollArea className="border-t pt-1">
          <div className="flex max-h-[250px] flex-col gap-2">
            <div className="flex flex-col gap-2 p-2 pt-0 pb-2">
              <span className="text-sm opacity-55 dark:font-light">
                {t("Currently Viewing")}
              </span>
              {Object.entries(users).map(([_key, value]) => {
                return <CollaborationCard userName={value.userName} />;
              })}
            </div>
          </div>
        </ScrollArea>
      )}
    </div>
  );
};

export default CollaborationPopover;
