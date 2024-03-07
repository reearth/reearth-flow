import { FileIcon, StopIcon } from "@radix-ui/react-icons";

import BoilerFiletree from "@flow/assets/filetree-example.png";
import { CollapsibleSidebar, type SidebarContent } from "@flow/components";

type Props = {
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
};

const LeftPanel: React.FC<Props> = ({ isSidebarOpen, toggleSidebar }) => {
  const sidebarContents: SidebarContent[] = [
    {
      id: "navigator",
      title: "Navigator",
      icon: <FileIcon />,
      component: (
        <div className="flex gap-1 items-center mb-4">
          <img className="opacity-50" src={BoilerFiletree} alt="file-tree-example" />
        </div>
      ),
    },
    {
      id: "transformer-gallery",
      icon: <StopIcon />,
      title: "Transformers Gallery",
      component: (
        <div>
          <p className="text-sm">All of my transformers</p>
          <p className="text-sm">All of my transformers</p>
          <p className="text-sm">All of my transformers</p>
          <p className="text-sm">All of my transformers</p>
        </div>
      ),
    },
  ];
  return (
    <CollapsibleSidebar
      className="bg-zinc-800 ml-1 mb-1 mr-1 rounded-md text-zinc-300"
      isOpen={isSidebarOpen}
      togglePosition="end"
      toggleSidebar={toggleSidebar}
      sidebarContents={sidebarContents}
    />
  );
};

export default LeftPanel;
