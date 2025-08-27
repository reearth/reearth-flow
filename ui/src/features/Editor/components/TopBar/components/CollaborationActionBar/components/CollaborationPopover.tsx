import { ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";

import CollaborationCard from "./CollaborationCard";

type Props = {
  users?: any;
};

const CollaborationPopover: React.FC<Props> = ({ users }) => {
  const t = useT();

  return (
    <div className="flex flex-col gap-2">
      <div className="border-b p-2">
        <CollaborationCard user={users[0]} />
      </div>
      <ScrollArea>
        <div className="flex max-h-[250px] flex-col gap-2">
          <div className="flex flex-col gap-2 p-2 pt-0 pb-2">
            <span className="text-sm opacity-55 dark:font-light">
              {t("Currently Viewing")}
            </span>
            {users.slice(1, 3).map((user: any, index: number) => (
              <CollaborationCard user={user} key={index} />
            ))}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
};

export default CollaborationPopover;
