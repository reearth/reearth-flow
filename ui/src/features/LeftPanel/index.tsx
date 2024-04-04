import { FileIcon, StopIcon } from "@radix-ui/react-icons";

import BoilerFiletree from "@flow/assets/filetree-example.png";
import { VerticalPanel, FlowLogo, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { useT } from "@flow/providers";

import HomeMenu from "../HomeMenu";

type Props = {
  className?: string;
};

const LeftPanel: React.FC<Props> = ({ className }) => {
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(true);
  const t = useT();

  const panelContents: PanelContent[] = [
    {
      id: "home-menu",
      icon: <FlowLogo />,
      component: (
        <>
          <HomeMenu />
          <div className="border-zinc-700 border-t-[1px] w-[100%]" />
        </>
      ),
    },
    {
      id: "navigator",
      title: t("Navigator"),
      icon: <FileIcon />,
      component: <img src={BoilerFiletree} alt="file-tree-example" />,
    },
    {
      id: "transformer-gallery",
      icon: <StopIcon />,
      title: t("Transformers Gallery"),
      component: (
        <div>
          <p className="text-xs">{t("All of my transformers")}</p>
          <p className="text-xs">{t("All of my transformers")}</p>
          <p className="text-xs">{t("All of my transformers")}</p>
          <p className="text-xs">{t("All of my transformers")}</p>
        </div>
      ),
    },
  ];
  return (
    <VerticalPanel
      className={`bg-zinc-800 bg-opacity-75 rounded-md backdrop-blur-md ${className}`}
      isOpen={!!isPanelOpen}
      togglePosition="end-right"
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default LeftPanel;
