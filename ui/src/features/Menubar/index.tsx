import {
  DoubleArrowRightIcon,
  EnterFullScreenIcon,
  ExitFullScreenIcon,
  Link2Icon,
  PlayIcon,
  ZoomInIcon,
  ZoomOutIcon,
} from "@radix-ui/react-icons";

import { Button, FlowLogo, Menubar, MenubarSeparator } from "@flow/components";
import { closeFullscreen, openFullscreen } from "@flow/utils";

import EditMenu from "./components/Edit";
import HelpMenu from "./components/Help";
import RunMenu from "./components/Run";
import ToolsMenu from "./components/Tools";
import ViewMenu from "./components/View";

export default function MenubarComponent() {
  return (
    <Menubar className="border-none bg-zinc-800 m-1">
      <Button
        className="bg-red-900 h-[30px] w-[30px] border border-black"
        size="icon"
        variant="ghost">
        <FlowLogo />
      </Button>
      <EditMenu />
      <ViewMenu />
      <RunMenu />
      <ToolsMenu />
      <HelpMenu />
      <div className="flex justify-end align-middle gap-[10px] flex-1">
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <EnterFullScreenIcon onClick={openFullscreen} />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <ExitFullScreenIcon onClick={closeFullscreen} />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <ZoomInIcon />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <ZoomOutIcon />
        </Button>
        <MenubarSeparator />
        <div className="border-l border-zinc-700" />
        <MenubarSeparator />
      </div>
      <div className="flex justify-end align-middle gap-[10px]">
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <DoubleArrowRightIcon />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <PlayIcon />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <Link2Icon />
        </Button>
      </div>
    </Menubar>
  );
}
