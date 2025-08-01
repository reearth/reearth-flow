import {
  CornersInIcon,
  CornersOutIcon,
  FrameCornersIcon,
  MinusIcon,
} from "@phosphor-icons/react";
import { PlusIcon } from "@phosphor-icons/react/dist/ssr";
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
    <div className="pointer-events-auto rounded-md p-1">
      <div className="flex rounded-md">
        <div className="flex flex-1 flex-col justify-end gap-1 align-middle">
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("Zoom in")}
            tooltipPosition="left"
            tooltipOffset={tooltipOffset}
            icon={<PlusIcon size={18} weight="light" />}
            onClick={() => zoomIn({ duration: 400 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("Zoom out")}
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={<MinusIcon size={18} weight="light" />}
            onClick={() => zoomOut({ duration: 400 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("All nodes in viewport")}
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={<FrameCornersIcon weight="thin" size={18} />}
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
                <CornersInIcon weight="thin" size={18} />
              ) : (
                <CornersOutIcon weight="thin" size={18} />
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
