import {
  DoubleArrowRightIcon,
  DownloadIcon,
  EnterFullScreenIcon,
  ExitFullScreenIcon,
  Link2Icon,
  PlayIcon,
  ZoomInIcon,
  ZoomOutIcon,
} from "@radix-ui/react-icons";
import { useCallback, useState } from "react";
import { useReactFlow } from "reactflow";

import { CenterIcon, IconButton, Menubar } from "@flow/components";
import { checkIsFullscreen, closeFullscreen, openFullscreen } from "@flow/utils";

const tooltipOffset = 6;

export default function ActionBar() {
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const [isFullscreen, setIsFullscreen] = useState(false);

  const handleFullscreenToggle = useCallback(() => {
    const isFullscreen = checkIsFullscreen();
    if (isFullscreen) {
      closeFullscreen();
    } else {
      openFullscreen();
    }
    setIsFullscreen(!isFullscreen);
  }, []);

  return (
    <Menubar className="border-none bg-zinc-800 m-1">
      <div className="flex justify-end align-middle flex-1">
        <IconButton
          tooltipText={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
          tooltipOffset={tooltipOffset}
          icon={isFullscreen ? <ExitFullScreenIcon /> : <EnterFullScreenIcon />}
          onClick={handleFullscreenToggle}
        />
        <IconButton
          tooltipText="Zoom out"
          tooltipOffset={tooltipOffset}
          icon={<ZoomOutIcon />}
          onClick={() => zoomOut({ duration: 400 })}
        />
        <IconButton
          tooltipText="Zoom in"
          tooltipOffset={tooltipOffset}
          icon={<ZoomInIcon />}
          onClick={() => zoomIn({ duration: 400 })}
        />
        <IconButton
          tooltipText="All nodes in viewport"
          tooltipOffset={tooltipOffset}
          icon={<CenterIcon />}
          onClick={() => fitView({ duration: 400, padding: 0.5 })}
        />
        <div className="border-l border-zinc-700 mx-3" />
      </div>
      <div className="flex align-middle">
        <IconButton
          tooltipText="Incrementally run workflow"
          tooltipOffset={tooltipOffset}
          icon={<DoubleArrowRightIcon />}
        />
        <IconButton tooltipText="Run workflow" tooltipOffset={tooltipOffset} icon={<PlayIcon />} />
        <div className="border-l border-zinc-700 mx-3" />
      </div>
      <div className="flex align-middle">
        <IconButton
          tooltipText="Publish workflow"
          tooltipOffset={tooltipOffset}
          icon={<Link2Icon />}
        />
        <IconButton
          tooltipText="Download workflow"
          tooltipOffset={tooltipOffset}
          icon={<DownloadIcon />}
        />
      </div>
    </Menubar>
  );
}
