import {
  EnterFullScreenIcon,
  ExitFullScreenIcon,
  ZoomInIcon,
  ZoomOutIcon,
} from "@radix-ui/react-icons";
import { useReactFlow } from "reactflow";

import { CenterIcon, IconButton } from "@flow/components";
import { useFullscreen } from "@flow/hooks";
import { useT } from "@flow/providers";

const tooltipOffset = 6;

const CanvasActionBar = () => {
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { isFullscreen, handleFullscreenToggle } = useFullscreen();

  return (
    <div className="absolute bottom-2 right-2">
      <div className="bg-zinc-800 rounded-md border border-zinc-700 m-1">
        <div className="flex rounded-md bg-zinc-900/50 p-1">
          <div className="flex flex-col justify-end align-middle flex-1">
            <IconButton
              className="w-[30px] h-[30px]"
              tooltipText={t("Zoom in")}
              tooltipOffset={tooltipOffset}
              icon={<ZoomInIcon />}
              onClick={() => zoomIn({ duration: 400 })}
            />
            <IconButton
              className="w-[30px] h-[30px]"
              tooltipText={t("Zoom out")}
              tooltipOffset={tooltipOffset}
              icon={<ZoomOutIcon />}
              onClick={() => zoomOut({ duration: 400 })}
            />
            <IconButton
              className="w-[30px] h-[30px]"
              tooltipText={t("All nodes in viewport")}
              tooltipOffset={tooltipOffset}
              icon={<CenterIcon />}
              onClick={() => fitView({ duration: 400, padding: 0.5 })}
            />
            <IconButton
              className="w-[30px] h-[30px]"
              tooltipText={isFullscreen ? t("Exit fullscreen") : t("Enter fullscreen")}
              tooltipOffset={tooltipOffset}
              icon={isFullscreen ? <ExitFullScreenIcon /> : <EnterFullScreenIcon />}
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
