import { DoubleArrowRightIcon, Link2Icon, PlayIcon, StopIcon } from "@radix-ui/react-icons";

import { Button } from "@flow/components";
import { Menubar } from "@flow/components/menubar";

import Github from "../../stories_examples/assets/github.svg";

import EditMenu from "./components/Edit";
import FileMenu from "./components/File";
import HelpMenu from "./components/Help";
import ReadersMenu from "./components/Readers";
import RunMenu from "./components/Run";
import ToolsMenu from "./components/Tools";
import TransformersMenu from "./components/Transformers";
import ViewMenu from "./components/View";
import WritersMenu from "./components/Writers";

export default function MenubarComponent() {
  return (
    <Menubar className="border-none bg-zinc-800 m-1 p-2" style={{ color: "#dbdbdb" }}>
      <Button
        className="bg-red-900 h-[30px] w-[30px] border border-black"
        size="icon"
        variant="ghost">
        <img src={Github} alt="Github" width="25px" height="25px" />
      </Button>
      <p className="text-xl pl-2 pr-4">Flow</p>
      <FileMenu />
      <EditMenu />
      <ViewMenu />
      <ReadersMenu />
      <TransformersMenu />
      <WritersMenu />
      <RunMenu />
      <ToolsMenu />
      <HelpMenu />
      <div className="flex justify-end align-middle gap-[10px] flex-1">
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <StopIcon />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <DoubleArrowRightIcon />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <PlayIcon />
        </Button>
        <Button className="hover:bg-zinc-600" variant="ghost" size="sm">
          <Link2Icon />
        </Button>
        {/* <Button className="bg-zinc-900 border border-zinc-600" size="sm">
          Publish
        </Button> */}
      </div>
    </Menubar>
  );
}
