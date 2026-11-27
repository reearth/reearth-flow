import { FrameCornersIcon, MinusIcon, PlusIcon } from "@phosphor-icons/react";
import { useReactFlow, useViewport } from "@xyflow/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

const EmbedZoomControls: React.FC = () => {
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { zoom } = useViewport();

  return (
    <div className="flex items-center gap-1 rounded-md border bg-secondary/70 p-1 shadow-md backdrop-blur-xs select-none">
      <IconButton
        className="size-6 rounded-[4px]"
        tooltipText={t("Zoom out")}
        icon={<MinusIcon size={14} weight="light" />}
        onClick={() => zoomOut({ duration: 300 })}
      />
      <span className="w-10 text-center text-xs tabular-nums">
        {Math.round(zoom * 100)}%
      </span>
      <IconButton
        className="size-6 rounded-[4px]"
        tooltipText={t("Zoom in")}
        icon={<PlusIcon size={14} weight="light" />}
        onClick={() => zoomIn({ duration: 300 })}
      />
      <div className="h-4 w-px bg-border" />
      <IconButton
        className="size-6 rounded-[4px]"
        tooltipText={t("All actions in viewport")}
        icon={<FrameCornersIcon size={14} weight="light" />}
        onClick={() => fitView({ duration: 300, padding: 0.5 })}
      />
    </div>
  );
};

export default memo(EmbedZoomControls);
