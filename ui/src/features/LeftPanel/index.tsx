import { FileIcon, StopIcon } from "@radix-ui/react-icons";

import BoilerFiletree from "@flow/assets/filetree-example.png";
import { CollapsiblePanel, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";

type Props = {};

const LeftPanel: React.FC<Props> = () => {
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(true);

  const panelContents: PanelContent[] = [
    {
      id: "navigator",
      title: "Navigator",
      icon: <FileIcon />,
      component: <img className="opacity-50 " src={BoilerFiletree} alt="file-tree-example" />,
    },
    {
      id: "transformer-gallery",
      icon: <StopIcon />,
      title: "Transformers Gallery",
      component: (
        <div>
          <p className="text-xs">All of my transformers</p>
          <p className="text-xs">All of my transformers</p>
          <p className="text-xs">All of my transformers</p>
          <p className="text-xs">All of my transformers</p>
        </div>
      ),
    },
  ];
  return (
    <CollapsiblePanel
      className="bg-zinc-800 ml-1 mb-1 mr-1 rounded-md"
      isOpen={!!isPanelOpen}
      togglePosition="end"
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default LeftPanel;
