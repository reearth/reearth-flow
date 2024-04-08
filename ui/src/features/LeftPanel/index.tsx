import { FileIcon, StopIcon } from "@radix-ui/react-icons";
import { Database, Folder } from "lucide-react";
import { useState } from "react";

import { VerticalPanel, FlowLogo, type PanelContent, Tree } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { useT } from "@flow/providers";

import HomeMenu from "./components/HomeMenu";
import { data } from "./MOCK_DATA"; // TODO: replace with real data

type Props = {
  className?: string;
};

const LeftPanel: React.FC<Props> = ({ className }) => {
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(true);
  const t = useT();

  const [_content, setContent] = useState("Admin Page");

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
      component: (
        <>
          <Tree
            data={data}
            className="flex-shrink-0 w-full h-[60vh] text-zinc-300"
            // initialSlelectedItemId="1"
            onSelectChange={item => setContent(item?.name ?? "")}
            folderIcon={Folder}
            itemIcon={Database}
          />
          <div className="border-zinc-700 border-t-[1px] w-[100%]" />
        </>
      ),
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
