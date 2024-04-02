import { FlowLogo } from "@flow/components";
import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarSub,
  MenubarSubContent,
  MenubarSubTrigger,
  MenubarTrigger,
} from "@flow/components/Menubar";

const HomeMenu: React.FC = () => {
  return (
    <MenubarMenu>
      <MenubarTrigger className="bg-red-900 hover:bg-red-800 transition-colors">
        <FlowLogo />
      </MenubarTrigger>
      <MenubarContent className="w-[300px] bg-zinc-800 border-none text-zinc-200">
        <MenubarItem>
          New Tab <MenubarShortcut>⌘T</MenubarShortcut>
        </MenubarItem>
        <MenubarItem>
          New Window <MenubarShortcut>⌘N</MenubarShortcut>
        </MenubarItem>
        <MenubarItem disabled>New Incognito Window</MenubarItem>
        <MenubarSeparator className="bg-zinc-700" />
        <MenubarSub>
          <MenubarSubTrigger>Share</MenubarSubTrigger>
          <MenubarSubContent>
            <MenubarItem>Email link</MenubarItem>
            <MenubarItem>Messages</MenubarItem>
            <MenubarItem>Notes</MenubarItem>
          </MenubarSubContent>
        </MenubarSub>
        <MenubarSeparator className="bg-zinc-700" />
        <MenubarItem>
          Print... <MenubarShortcut>⌘P</MenubarShortcut>
        </MenubarItem>
      </MenubarContent>
    </MenubarMenu>
  );
};

export default HomeMenu;
