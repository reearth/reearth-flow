import { CornersIn, CornersOut, Globe, Terminal } from "@phosphor-icons/react";
import { memo, useCallback, useState } from "react";

import { IconButton } from "@flow/components";
import { useShortcuts } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

import { WorkflowTabs } from "..";

import { DataTable, LogConsole, Map } from "./components";

type Props = {
  currentWorkflowId?: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowAdd: () => void;
  onWorkflowChange: (workflowId?: string) => void;
};

type PanelContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
  icon?: React.ReactNode;
};

type WindowSize = "min" | "max";

const BottomPanel: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  isOpen,
  onOpen,
  onWorkflowClose,
  onWorkflowAdd,
  onWorkflowChange,
}) => {
  const t = useT();
  const [windowSize, setWindowSize] = useState<WindowSize>("min");

  const handlePanelToggle = useCallback(
    (open: boolean) => onOpen(open ? "bottom" : undefined),
    [onOpen],
  );

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

  const [selectedId, setSelectedId] = useState<string>(panelContents?.[0].id);

  const handleSelection = useCallback(
    (id: string) => {
      if (id !== selectedId) {
        setSelectedId(id);
        if (!isOpen) {
          handlePanelToggle?.(true);
        }
      } else {
        handlePanelToggle?.(!isOpen);
      }
    },
    [isOpen, handlePanelToggle, selectedId, setSelectedId],
  );

  useShortcuts([
    {
      keyBinding: { key: "l", commandKey: true },
      callback: () => {
        handleSelection("output-log");
      },
    },
    {
      keyBinding: { key: "p", commandKey: true },
      callback: () => {
        handleSelection("visual-preview");
      },
    },
  ]);

  return (
    <div
      className="box-content flex flex-col justify-end border-t bg-secondary backdrop-blur-md duration-300 ease-in-out"
      style={{
        height: isOpen
          ? windowSize === "max"
            ? "calc(100vh - 1px)"
            : "50vh"
          : "29px",
      }}
    >
      {isOpen && (
        <div
          id="top-edge"
          className="flex h-[29px] shrink-0 items-center gap-1"
        >
          <div className="flex h-full flex-1 items-center justify-end gap-1 px-1">
            <BaseActionButtons
              panelContents={panelContents}
              selectedId={selectedId}
              onSelection={handleSelection}
            />
            {isOpen && (
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
        className={`flex h-[calc(100%-64px)] flex-1 bg-background ${isOpen ? "flex" : "hidden"}`}
      >
        {panelContents.map((p) => (
          <div
            className={`flex-1 ${selectedId === p.id ? "flex" : "hidden"}`}
            key={p.id}
          >
            {p.component}
          </div>
        ))}
      </div>
      <div
        id="bottom-edge"
        className="flex h-[29px] shrink-0 items-center justify-end gap-1 bg-secondary"
      >
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowAdd={onWorkflowAdd}
          onWorkflowChange={onWorkflowChange}
        />
        <div className="h-full border-r" />
        <div className="mx-4 flex h-full flex-1 items-center justify-end gap-1">
          {!isOpen && (
            <BaseActionButtons
              panelContents={panelContents}
              selectedId={selectedId}
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
  selectedId?: string;
  onSelection?: (id: string) => void;
}> = memo(({ panelContents, selectedId, onSelection }) => {
  return (
    <>
      {panelContents?.map((content) => (
        <div
          key={content.id}
          className={`flex h-4/5 min-w-[100px] cursor-pointer items-center justify-center gap-2 rounded hover:bg-popover hover:text-popover-foreground ${
            selectedId === content.id
              ? "bg-popover text-popover-foreground"
              : ""
          }`}
          onClick={() => onSelection?.(content.id)}
        >
          {content.icon}
          <p className="text-sm font-thin">{content.title}</p>
        </div>
      ))}
    </>
  );
});

BaseActionButtons.displayName = "BaseActionButtons";
