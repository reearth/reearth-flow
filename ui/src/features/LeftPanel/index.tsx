import { FileIcon, StopIcon } from "@radix-ui/react-icons";

import BoilerFiletree from "@flow/assets/filetree-example.png";
import { VerticalPanel, FlowLogo, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";

import HomeNav from "../HomeNav";

type Props = {
  className?: string;
};

const LeftPanel: React.FC<Props> = ({ className }) => {
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(true);

  const panelContents: PanelContent[] = [
    {
      id: "home-menu",
      icon: <FlowLogo />,
      component: (
        <>
          <HomeNav />
          <div className="border-zinc-700 border-t-[1px] w-[100%]" />
        </>
      ),
    },
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
    <VerticalPanel
      className={`bg-zinc-800 rounded-md ${className}`}
      isOpen={!!isPanelOpen}
      togglePosition="end-right"
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default LeftPanel;
