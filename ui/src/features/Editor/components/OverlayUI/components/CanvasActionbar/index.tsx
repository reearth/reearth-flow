import {
  CornersIn,
  CornersOut,
  FrameCorners,
  MagnifyingGlassMinus,
  MagnifyingGlassPlus,
} from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";

import { IconButton } from "@flow/components";
import { useFullscreen } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

const tooltipOffset = 6;

const CanvasActionBar = () => {
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { isFullscreen, handleFullscreenToggle } = useFullscreen();

  return (
    <div className="absolute bottom-2 right-2">
      <div className="m-1 rounded-md border border-zinc-700 bg-zinc-800">
        <div className="flex rounded-md bg-zinc-900/50 p-1">
          <div className="flex flex-1 flex-col justify-end align-middle">
            <IconButton
              className="size-[30px]"
              tooltipText={t("Zoom in")}
              tooltipOffset={tooltipOffset}
              icon={<MagnifyingGlassPlus />}
              onClick={() => zoomIn({ duration: 400 })}
            />
            <IconButton
              className="size-[30px]"
              tooltipText={t("Zoom out")}
              tooltipOffset={tooltipOffset}
              icon={<MagnifyingGlassMinus />}
              onClick={() => zoomOut({ duration: 400 })}
            />
            <IconButton
              className="size-[30px]"
              tooltipText={t("All nodes in viewport")}
              tooltipOffset={tooltipOffset}
              icon={<FrameCorners />}
              onClick={() => fitView({ duration: 400, padding: 0.5 })}
            />
            <IconButton
              className="size-[30px]"
              tooltipText={isFullscreen ? t("Exit fullscreen") : t("Enter fullscreen")}
              tooltipOffset={tooltipOffset}
              icon={isFullscreen ? <CornersIn /> : <CornersOut />}
              onClick={handleFullscreenToggle}
            />
            {/* <div className="border-l border-zinc-700 mx-3" /> */}
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
    </div>
  );
};

export { CanvasActionBar };
