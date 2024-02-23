import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarCheckboxItem,
  MenubarTrigger,
} from "@flow/components/menubar";

const ViewMenu: React.FC = () => {
  return (
    <MenubarMenu>
      <MenubarTrigger>View</MenubarTrigger>
      <MenubarContent>
        <MenubarCheckboxItem>Always Show Bookmarks Bar</MenubarCheckboxItem>
        <MenubarCheckboxItem checked>Always Show Full URLs</MenubarCheckboxItem>
        <MenubarSeparator />
        <MenubarItem inset>
          Reload <MenubarShortcut>⌘R</MenubarShortcut>
        </MenubarItem>
        <MenubarItem disabled inset>
          Force Reload <MenubarShortcut>⇧⌘R</MenubarShortcut>
        </MenubarItem>
        <MenubarSeparator />
        <MenubarItem inset>Toggle Fullscreen</MenubarItem>
        <MenubarSeparator />
        <MenubarItem inset>Hide Sidebar</MenubarItem>
      </MenubarContent>
    </MenubarMenu>
  );
};

export default ViewMenu;
