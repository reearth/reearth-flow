import {
  DoubleArrowRightIcon,
  EnterFullScreenIcon,
  ExitFullScreenIcon,
  Link2Icon,
  PlayIcon,
  ZoomInIcon,
  ZoomOutIcon,
} from "@radix-ui/react-icons";
import { useCallback, useState } from "react";
import { useReactFlow } from "reactflow";

import { IconButton, Menubar, MenubarSeparator } from "@flow/components";
import { checkIsFullscreen, closeFullscreen, openFullscreen } from "@flow/utils";

export default function ActionBar() {
  const { zoomIn, zoomOut } = useReactFlow();
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
      <div className="flex justify-end align-middle gap-[10px] flex-1">
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
        <MenubarSeparator />
        <div className="border-l border-zinc-700" />
        <MenubarSeparator />
      </div>
      <div className="flex justify-end align-middle gap-[10px]">
        <IconButton icon={<DoubleArrowRightIcon />} />
        <IconButton icon={<PlayIcon />} />
        <IconButton icon={<Link2Icon />} />
      </div>
    </Menubar>
  );
}
