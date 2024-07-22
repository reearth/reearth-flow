import { CornersIn, CornersOut, Globe, Terminal } from "@phosphor-icons/react";
import { memo, useCallback, useState } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { WorkflowTabs } from "..";

import { DataTable, LogConsole, Map } from "./components";

type Props = {
  currentWorkflowId?: string;
  onWorkflowChange: (workflowId?: string) => void;
};

type PanelContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
  icon?: React.ReactNode;
};

type WindowSize = "min" | "max";

const BottomPanel: React.FC<Props> = ({ currentWorkflowId, onWorkflowChange }) => {
  const t = useT();
  const [windowSize, setWindowSize] = useState<WindowSize>("min");
  const [isPanelOpen, setIsPanelOpen] = useState(false);

  const handlePanelToggle = useCallback((open: boolean) => setIsPanelOpen(open), []);

  const panelContents: PanelContent[] = [
    {
      id: "output-log",
      icon: <Terminal className="size-[20px]" weight="thin" />,
      title: t("Log"),
      component: <LogConsole />,
    },
    {
      id: "visual-preview",
      icon: <Globe className="size-[20px]" weight="thin" />,
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
      className="box-content flex flex-col justify-end border-t border-zinc-700 bg-zinc-800 backdrop-blur-md duration-300 ease-in-out"
      style={{
        height: isPanelOpen ? (windowSize === "max" ? "calc(100vh - 1px)" : "50vh") : "29px",
      }}>
      {isPanelOpen && (
        <div id="top-edge" className="flex h-[29px] shrink-0 items-center gap-1 bg-zinc-900/50">
          <div className="flex h-full flex-1 items-center justify-end gap-1 px-1">
            <BaseActionButtons
              panelContents={panelContents}
              selected={selected}
              onSelection={handleSelection}
            />
            {isPanelOpen && (
              <div className="flex h-[29px] items-center px-1">
                {windowSize === "min" && (
                  <IconButton
                    className="h-4/5 w-[55px]"
                    icon={<CornersOut />}
                    tooltipText={"Enter full screen"}
                    tooltipPosition="top"
                    onClick={() => setWindowSize("max")}
                  />
                )}
                {windowSize === "max" && (
                  <IconButton
                    className="h-4/5 w-[55px]"
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
        className={`flex h-[calc(100%-64px)] flex-1 bg-zinc-800 ${isPanelOpen ? "flex" : "hidden"}`}>
        {panelContents.map(p => (
          <div className={`flex-1 ${selected?.id === p.id ? "flex" : "hidden"}`} key={p.id}>
            {p.component}
          </div>
        ))}
      </div>
      <div
        id="bottom-edge"
        className="flex h-[29px] shrink-0 items-center justify-end gap-1 bg-zinc-900/50">
        <WorkflowTabs currentWorkflowId={currentWorkflowId} onWorkflowChange={onWorkflowChange} />
        <div className="h-full border-r border-zinc-700" />
        <div className="mx-4 flex h-full flex-1 items-center justify-end gap-1">
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

export default memo(BottomPanel);

const BaseActionButtons: React.FC<{
  panelContents?: PanelContent[];
  selected?: PanelContent;
  onSelection?: (content: PanelContent) => void;
}> = memo(({ panelContents, selected, onSelection }) => {
  return (
    <>
      {panelContents?.map(content => (
        <div
          key={content.id}
          className={`flex h-4/5 min-w-[100px] cursor-pointer items-center justify-center gap-2 rounded hover:bg-zinc-700/75 hover:text-white ${
            selected?.id === content.id ? "bg-zinc-700/75 text-white" : ""
          }`}
          onClick={() => onSelection?.(content)}>
          {content.icon}
          <p className="text-sm font-thin">{content.title}</p>
        </div>
      ))}
    </>
  );
});

BaseActionButtons.displayName = "BaseActionButtons";
