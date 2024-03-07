import { CollapsibleSidebar, type SidebarContent } from "@flow/components";

export type BottomPanelProps = {
  isBottomBarOpen: boolean;
  toggleBottombar: () => void;
};

const BottomPanel: React.FC<BottomPanelProps> = ({ isBottomBarOpen, toggleBottombar }) => {
  const sidebarContents: SidebarContent[] = [
    {
      id: "1",
      component: (
        <div className="flex justify-between h-full" style={{ color: "#dbdbdb" }}>
          <div className="flex gap-6 ml-[18px]">
            <p className="text-sm">Translation Log</p>
            <p className="text-sm">Visual Preview</p>
          </div>
          <div className="self-end">
            <p className="text-xs">yokohama_river.proj - CDED -&#62; NONE - Flow 2024 &#169;</p>
          </div>
        </div>
      ),
    },
  ];

  return (
    <CollapsibleSidebar
      className="bg-zinc-950 mr-1 mb-1 rounded-md"
      direction="horizontal"
      isOpen={isBottomBarOpen}
      minHeight="h-[25px]"
      maxHeight="h-[200px]"
      toggleSidebar={toggleBottombar}
      sidebarContents={sidebarContents}
    />
  );
};

export default BottomPanel;
