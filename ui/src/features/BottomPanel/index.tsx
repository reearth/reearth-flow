import { CornersIn, CornersOut, Globe, Terminal } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import { IconButton } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { customTransformers } from "@flow/mock_data/customTransformer";

import { WorkflowTabs } from "../Editor/components";

import { DataTable, LogConsole, Map } from "./components";

type PanelContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
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
      title: t("Log"),
      component: <LogConsole />,
    },
    {
      id: "visual-preview",
      icon: <Globe className="w-[20px] h-[20px]" weight="thin" />,
      title: t("Preview"),
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
      className="flex flex-col justify-end box-content transition-width duration-300 ease-in-out bg-zinc-800 border-t border-zinc-700 backdrop-blur-md"
      style={{
        height: isPanelOpen ? (windowSize === "max" ? "calc(100vh - 1px)" : "50vh") : "29px",
      }}>
      {isPanelOpen && (
        <div id="top-edge" className="flex gap-1 items-center shrink-0 h-[29px] bg-zinc-900/50">
          <div className="flex justify-end gap-1 px-1 items-center flex-1 h-[100%]">
            <BaseActionButtons
              panelContents={panelContents}
              selected={selected}
              onSelection={handleSelection}
            />
            {isPanelOpen && (
              <div className="h-[29px] flex items-center px-1">
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
        </div>
      )}
      <div
        id="content"
        className={`flex flex-1 h-[calc(100%-64px)] bg-zinc-800 ${isPanelOpen ? "flex" : "hidden"}`}>
        {panelContents.map(p => (
          <div className={`flex-1 ${selected?.id === p.id ? "flex" : "hidden"}`} key={p.id}>
            {p.component}
          </div>
        ))}
      </div>
      <div
        id="bottom-edge"
        className="flex gap-1 justify-end items-center shrink-0 h-[29px] bg-zinc-900/50">
        <WorkflowTabs editingCustomTransformers={customTransformers} />
        <div className="border-r border-zinc-700 h-full" />
        <div className="flex justify-end items-center gap-1 flex-1 h-[100%] mx-4">
          {!isPanelOpen && (
            <BaseActionButtons
              panelContents={panelContents}
              selected={selected}
              onSelection={handleSelection}
            />
          )}
        </div>
      </div>
    </div>
  );
};

export default BottomPanel;

const BaseActionButtons: React.FC<{
  panelContents?: PanelContent[];
  selected?: PanelContent;
  onSelection?: (content: PanelContent) => void;
}> = ({ panelContents, selected, onSelection }) => {
  return panelContents?.map(content => (
    <div
      key={content.id}
      className={`flex justify-center items-center gap-2 h-[80%] min-w-[100px] rounded cursor-pointer hover:bg-zinc-700/75 hover:text-white ${selected?.id === content.id ? "text-white bg-zinc-700/75" : undefined}`}
      onClick={() => onSelection?.(content)}>
      {content.icon}
      <p className="text-sm font-thin">{content.title}</p>
    </div>
  ));
};
