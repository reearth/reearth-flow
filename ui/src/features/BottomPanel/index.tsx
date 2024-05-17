import { EnterFullScreenIcon, ExitFullScreenIcon } from "@radix-ui/react-icons";
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

type WindowSize = "min" | "max";

const BottomPanel: React.FC = () => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);
  const t = useT();
  const [windowSize, setWindowSize] = useState<WindowSize>("min");

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
        <div className={`flex flex-1`}>
          <DataTable />
          <Map />
        </div>
      ),
    },
  ];

  const [selected, setSelected] = useState<PanelContent | undefined>(panelContents?.[0]);

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
    <div
      // className={`flex flex-col box-content transition-width duration-300 ease-in-out bg-zinc-900 border-t border-zinc-700 backdrop-blur-md ${isPanelOpen ? (windowSize === "max" ? "h-full" : "h-[400px]") : "h-[36px]"}`}>
      className={`flex flex-col box-content transition-width duration-300 ease-in-out bg-zinc-900 border-t border-zinc-700 backdrop-blur-md`}
      style={{
        height: isPanelOpen ? (windowSize === "max" ? "100vh" : "400px") : "36px",
      }}>
      <div id="edge" className="flex gap-1 items-center h-[36px]">
        <div className="flex gap-1 items-center justify-center flex-1 h-[100%]">
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
        {windowSize === "min" && (
          <IconButton
            className={`w-[55px] h-[80%]`}
            icon={<EnterFullScreenIcon />}
            tooltipText={"Enter full screen"}
            tooltipPosition="top"
            onClick={() => setWindowSize("max")}
          />
        )}
        {windowSize === "max" && (
          <IconButton
            className={`w-[55px] h-[80%]`}
            icon={<ExitFullScreenIcon />}
            tooltipText={"Enter full screen"}
            tooltipPosition="top"
            onClick={() => setWindowSize("min")}
          />
        )}
      </div>
      <div
        id="content"
        className={`flex flex-1 bg-zinc-800}`}
        style={{
          height: isPanelOpen
            ? windowSize === "max"
              ? "calc(100vh - 36px)"
              : "calc(400px - 36px)"
            : "0",
        }}>
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
