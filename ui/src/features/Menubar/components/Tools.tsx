import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarTrigger,
} from "@flow/components/menubar";

const ToolsMenu: React.FC = () => {
  return (
    <MenubarMenu>
      <MenubarTrigger>Tools</MenubarTrigger>
      <MenubarContent>
        <MenubarRadioGroup value="benoit">
          <MenubarRadioItem value="andy">Andy</MenubarRadioItem>
          <MenubarRadioItem value="benoit">Benoit</MenubarRadioItem>
          <MenubarRadioItem value="Luis">Luis</MenubarRadioItem>
        </MenubarRadioGroup>
        <MenubarSeparator />
        <MenubarItem inset>Edit...</MenubarItem>
        <MenubarSeparator />
        <MenubarItem inset>Add Profile...</MenubarItem>
      </MenubarContent>
    </MenubarMenu>
  );
};

export default ToolsMenu;
