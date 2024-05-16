import { useCallback, useState } from "react";

import { OutputIcon, PreviewIcon, IconButton } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { useT } from "@flow/providers";

import { DataTable, LogConsole, Map } from "./components";

type PanelContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
  description?: string;
  icon?: React.ReactNode;
};

const BottomPanel: React.FC = () => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);
  const t = useT();

  const panelContents: PanelContent[] = [
    {
      id: "output-log",
      icon: <OutputIcon />,
      description: t("Output log"),
      component: <LogConsole />,
    },
    {
      id: "visual-preview",
      icon: <PreviewIcon />,
      description: t("Preview data"),
      component: (
        <div className="flex flex-1 h-[400px]">
          <DataTable />
          <Map />
        </div>
      ),
    },
  ];

  const [selected, setSelected] = useState<PanelContent | undefined>(panelContents?.[0]);

  const baseClasses = "flex flex-col box-content transition-width duration-300 ease-in-out";
  const classes = [
    baseClasses,
    isPanelOpen ? "h-100" : "h-[36px]",
    "bg-zinc-900 border-t border-zinc-700 backdrop-blur-md",
  ].reduce((acc, cur) => (cur ? `${acc} ${cur}` : acc));

  const handleSelection = useCallback(
    (content: PanelContent) => {
      if (content.id !== selected?.id) {
        setSelected(content);
        if (!isPanelOpen) {
          handlePanelToggle?.(true);
        }
      } else {
        handlePanelToggle?.(!isPanelOpen);
        if (content.id === selected?.id) {
          setSelected(undefined);
        }
      }
    },
    [isPanelOpen, handlePanelToggle, selected],
  );

  return (
    <div className={classes}>
      <div className="flex gap-1 items-center justify-center h-[36px]">
        {panelContents?.map(content => (
          <IconButton
            key={content.id}
            className={`w-[55px] h-[80%] ${selected?.id === content.id ? "text-white bg-zinc-800" : undefined}`}
            icon={content.icon}
            tooltipText={content.description}
            tooltipPosition="top"
            onClick={() => handleSelection(content)}
          />
        ))}
      </div>
      <div id="content" className={`flex flex-1 bg-zinc-800}`}>
        {panelContents.map(p => (
          <div className={`flex-1 p-1 ${selected?.id === p.id ? "flex" : "hidden"}`} key={p.id}>
            {p.component}
          </div>
        ))}
      </div>
    </div>
  );
};

export default BottomPanel;
