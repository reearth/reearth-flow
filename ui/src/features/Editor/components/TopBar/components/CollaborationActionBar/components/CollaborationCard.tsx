import { ProhibitIcon, TargetIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  self?: boolean;
  clientId: number;
  userName: string;
  color: string;
  spotlightUserClientId?: number | null;
  onSpotlightUserSelect?: (clientId: number) => void;
  onSpotlightUserDeselect?: () => void;
};

const CollaborationCard: React.FC<Props> = ({
  self,
  clientId,
  userName,
  color,
  spotlightUserClientId,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
}) => {
  const isSpotlighted = spotlightUserClientId === clientId;
  const t = useT();
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div
      className="flex items-center gap-2 p-1 hover:bg-primary"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}>
      <div
        className="flex h-10 w-10 items-center justify-center rounded-full ring-2 ring-secondary/20 "
        style={{ backgroundColor: color }}>
        <span className="text-sm font-medium">
          {userName.charAt(0).toUpperCase()}
        </span>
      </div>
      <div className="flex-grow">
        <span className="text-sm dark:font-light">{userName}</span>
      </div>
      {isHovered && !isSpotlighted && !self && (
        <IconButton
          tooltipText={t("Spotlight User")}
          icon={<TargetIcon size={16} />}
          onClick={() => onSpotlightUserSelect?.(clientId)}
        />
      )}
      {isSpotlighted && !self && (
        <IconButton
          tooltipText={t("Remove Spotlight")}
          icon={<ProhibitIcon size={16} />}
          onClick={() => onSpotlightUserDeselect?.()}
        />
      )}
    </div>
  );
};

export default CollaborationCard;
