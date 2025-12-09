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
      className="flex items-center gap-2 rounded-lg p-1 hover:bg-primary"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}>
      <div
        className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full ring-2 ring-secondary/20"
        style={{ backgroundColor: color }}>
        <span className="text-sm font-medium text-white select-none">
          {userName.charAt(0).toUpperCase()}
          {userName.charAt(1)}
        </span>
      </div>
      <div className="flex-grow">
        <span className="text-sm select-none dark:font-light">{userName}</span>
      </div>
      {isHovered && !isSpotlighted && !self && (
        <IconButton
          className="h-8"
          tooltipText={t("Spotlight User")}
          icon={<TargetIcon size={14} />}
          onClick={() => onSpotlightUserSelect?.(clientId)}
        />
      )}
      {isSpotlighted && !self && (
        <IconButton
          className="h-8"
          tooltipText={t("Remove Spotlight")}
          icon={<ProhibitIcon size={14} />}
          onClick={() => onSpotlightUserDeselect?.()}
        />
      )}
    </div>
  );
};

export default CollaborationCard;
