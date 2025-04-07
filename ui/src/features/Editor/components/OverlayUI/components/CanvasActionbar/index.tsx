import {
  CornersIn,
  CornersOut,
  FrameCorners,
  MagnifyingGlassMinus,
  MagnifyingGlassPlus,
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
    <div className="pointer-events-auto rounded-md bg-secondary p-1">
      <div className="flex rounded-md">
        <div className="flex flex-1 flex-col justify-end align-middle gap-1">
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("Zoom in")}
            tooltipPosition="left"
            tooltipOffset={tooltipOffset}
            icon={<MagnifyingGlassPlus weight="thin" />}
            onClick={() => zoomIn({ duration: 400 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("Zoom out")}
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={<MagnifyingGlassMinus weight="thin" />}
            onClick={() => zoomOut({ duration: 400 })}
          />
          <IconButton
            className="rounded-[4px]"
            tooltipText={t("All nodes in viewport")}
            tooltipOffset={tooltipOffset}
            tooltipPosition="left"
            icon={<FrameCorners weight="thin" />}
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
                <CornersIn weight="thin" />
              ) : (
                <CornersOut weight="thin" />
              )
            }
            onClick={handleFullscreenToggle}
          />
          {/* <div className="border-l  mx-3" /> */}
        </div>
        {/* <div className="flex align-middle">
          <IconButton
            tooltipText={t("Publish workflow")}
            tooltipOffset={tooltipOffset}
            icon={<Link2Icon />}
          />
          <IconButton
            tooltipText={t("Download workflow")}
            tooltipOffset={tooltipOffset}
            icon={<DownloadIcon />}
          />
        </div> */}
      </div>
    </div>
  );
};

export default memo(CanvasActionBar);
