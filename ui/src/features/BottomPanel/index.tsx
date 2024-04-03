import { HorizontalPanel, OutputIcon, PreviewIcon, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";

import { DataTable, LogConsole, TwoDMap } from "./components";

export type BottomPanelProps = {
  className?: string;
};

const BottomPanel: React.FC<BottomPanelProps> = ({ className }) => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);

  const panelContents: PanelContent[] = [
    {
      id: "translation-log",
      icon: <OutputIcon />,
      component: <LogConsole />,
    },
    {
      id: "visual-preview",
      icon: <PreviewIcon />,
      component: (
        <div className="flex flex-1 h-full">
          <DataTable />
          <TwoDMap className="flex-1" />
        </div>
      ),
    },
  ];

  // backdrop-filter: blur(10px);

  return (
    <HorizontalPanel
      className={`bg-zinc-950 rounded-tr-md cursor-pointer backdrop-blur-md ${className}`}
      isOpen={!!isPanelOpen}
      panelContents={panelContents}
      onToggle={handlePanelToggle}
    />
  );
};

export default BottomPanel;
