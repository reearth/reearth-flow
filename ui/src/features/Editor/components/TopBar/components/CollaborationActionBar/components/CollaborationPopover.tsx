import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser } from "@flow/types";

import CollaborationCard from "./CollaborationCard";

type Props = {
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
  spotlightUserClientId: number | null;
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
};

const CollaborationPopover: React.FC<Props> = ({
  self,
  users,
  spotlightUserClientId,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
}) => {
  const t = useT();

  return (
    <div className="flex flex-col gap-2">
      <div className="p-2">
        <CollaborationCard
          self
          clientId={self.clientId}
          cursor={self.cursor}
          userName={self?.userName}
          color={self.color}
        />
      </div>
      {users && Object.entries(users).length >= 1 && (
        <ScrollArea className="border-t pt-1">
          <div className="flex max-h-[250px] flex-col gap-2">
            <div className="flex flex-col gap-2 p-2 pt-0 pb-2">
              <span className="text-sm opacity-55 dark:font-light">
                {t("Currently Viewing")}
              </span>
              {Object.entries(users).map(([_key, value]) => {
                return (
                  <CollaborationCard
                    clientId={value.clientId}
                    cursor={value.cursor}
                    key={value.clientId}
                    userName={value.userName}
                    color={value.color}
                    spotlightUserClientId={spotlightUserClientId}
                    onSpotlightUserSelect={onSpotlightUserSelect}
                    onSpotlightUserDeselect={onSpotlightUserDeselect}
                  />
                );
              })}
            </div>
          </div>
        </ScrollArea>
      )}
    </div>
  );
};

export default CollaborationPopover;
