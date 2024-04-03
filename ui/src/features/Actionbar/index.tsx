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
          icon={
            isFullscreen ? (
              <ExitFullScreenIcon onClick={handleFullscreenToggle} />
            ) : (
              <EnterFullScreenIcon onClick={handleFullscreenToggle} />
            )
          }
        />
        <IconButton icon={<ZoomOutIcon />} onClick={() => zoomOut({ duration: 400 })} />
        <IconButton icon={<ZoomInIcon />} onClick={() => zoomIn({ duration: 400 })} />
        <IconButton
          icon={<CenterIcon />}
          onClick={() => fitView({ duration: 400, padding: 0.5 })}
        />
        <div className="border-l border-zinc-700 mx-3" />
      </div>
      <div className="flex align-middle">
        <IconButton icon={<DoubleArrowRightIcon />} />
        <IconButton icon={<PlayIcon />} />
        <div className="border-l border-zinc-700 mx-3" />
      </div>
      <div className="flex align-middle">
        <IconButton icon={<Link2Icon />} />
        <IconButton icon={<DownloadIcon />} />
      </div>
    </Menubar>
  );
}
