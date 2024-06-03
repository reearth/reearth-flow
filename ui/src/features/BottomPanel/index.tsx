import { CornersIn, CornersOut, Globe, Terminal } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import { IconButton } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

import { DataTable, LogConsole, Map } from "./components";

type PanelContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
  description?: string;
  icon?: React.ReactNode;
};

type WindowSize = "min" | "max";

const BottomPanel: React.FC = () => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);
  const t = useT();
  const [windowSize, setWindowSize] = useState<WindowSize>("min");

  const panelContents: PanelContent[] = [
    {
      id: "output-log",
      icon: <Terminal className="w-[20px] h-[20px]" weight="thin" />,
      description: t("Output log"),
      component: <LogConsole />,
    },
    {
      id: "visual-preview",
      icon: <Globe className="w-[20px] h-[20px]" weight="thin" />,
      description: t("Preview data"),
      component: (
        <div className="flex flex-1">
          <DataTable />
          <Map />
        </div>
      ),
    },
  ];

  const [selected, setSelected] = useState<PanelContent>(panelContents?.[0]);

  const handleSelection = useCallback(
    (content: PanelContent) => {
      if (content.id !== selected?.id) {
        setSelected(content);
        if (!isPanelOpen) {
          handlePanelToggle?.(true);
        }
      } else {
        handlePanelToggle?.(!isPanelOpen);
      }
    },
    [isPanelOpen, handlePanelToggle, selected],
  );

  return (
    <div
      className="flex flex-col box-content transition-width duration-300 ease-in-out bg-zinc-800 border-t border-zinc-700 backdrop-blur-md"
      style={{
        height: isPanelOpen ? (windowSize === "max" ? "100vh" : "50vh") : "36px",
      }}>
      <div id="edge" className="flex gap-1 items-center shrink-0 h-[36px] bg-zinc-900/50">
        <div className="flex gap-1 items-center justify-center flex-1 h-[100%]">
          {panelContents?.map(content => (
            <IconButton
              key={content.id}
              className={`w-[55px] h-[80%] ${selected?.id === content.id ? "text-white bg-zinc-700" : undefined}`}
              icon={content.icon}
              tooltipText={content.description}
              tooltipPosition="top"
              onClick={() => handleSelection(content)}
            />
          ))}
        </div>
        {isPanelOpen && (
          <div className="fixed right-0 h-[36px] flex items-center">
            {windowSize === "min" && (
              <IconButton
                className="w-[55px] h-[80%]"
                icon={<CornersOut />}
                tooltipText={"Enter full screen"}
                tooltipPosition="top"
                onClick={() => setWindowSize("max")}
              />
            )}
            {windowSize === "max" && (
              <IconButton
                className="w-[55px] h-[80%]"
                icon={<CornersIn />}
                tooltipText={"Enter full screen"}
                tooltipPosition="top"
                onClick={() => setWindowSize("min")}
              />
            )}
          </div>
        )}
      </div>
      <div
        id="content"
        className="flex flex-1 h-[calc(100%-36px)] bg-zinc-800"
        style={{
          display: isPanelOpen ? "flex" : "none",
        }}>
        {panelContents.map(p => (
          <div
            className="flex-1 p-1"
            style={{
              display: selected?.id === p.id ? "flex" : "none",
            }}
            key={p.id}>
            {p.component}
          </div>
        ))}
      </div>
    </div>
  );
};

export default BottomPanel;
