import {
  CornersInIcon,
  CornersOutIcon,
  FrameCornersIcon,
  MagnifyingGlassMinusIcon,
  MagnifyingGlassPlusIcon,
} from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useFullscreen } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

const tooltipOffset = 6;

const CanvasActionBar = () => {
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { isFullscreen, handleFullscreenToggle } = useFullscreen();

  return (
    <div className="pointer-events-auto rounded-md bg-secondary/80 p-1 backdrop-blur-xs shadow-md">
      <div className="flex rounded-md">
        <div className="flex flex-1 flex-col justify-end align-middle gap-1">
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("Zoom in")}
            tooltipPosition="left"
            tooltipOffset={tooltipOffset}
            icon={<MagnifyingGlassPlusIcon weight="thin" size={16} />}
            onClick={() => zoomIn({ duration: 400 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("Zoom out")}
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={<MagnifyingGlassMinusIcon weight="thin" size={16} />}
            onClick={() => zoomOut({ duration: 400 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("All nodes in viewport")}
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={<FrameCornersIcon weight="thin" size={16} />}
            onClick={() => fitView({ duration: 400, padding: 0.5 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={
              isFullscreen ? t("Exit fullscreen") : t("Enter fullscreen")
            }
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={
              isFullscreen ? (
                <CornersInIcon weight="thin" size={16} />
              ) : (
                <CornersOutIcon weight="thin" size={16} />
              )
            }
            onClick={handleFullscreenToggle}
          />
        </div>
      </div>
    </div>
  );
};

export default memo(CanvasActionBar);
