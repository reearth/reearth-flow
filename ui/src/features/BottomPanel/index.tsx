import { HorizontalPanel, OutputIcon, PreviewIcon, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { useT } from "@flow/providers";

import { DataTable, LogConsole, Map } from "./components";

export type BottomPanelProps = {
  className?: string;
};

const BottomPanel: React.FC<BottomPanelProps> = ({ className }) => {
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

  return (
    <HorizontalPanel
      className={`bg-zinc-900 border-t border-zinc-700 backdrop-blur-md ${className}`}
      isOpen={!!isPanelOpen}
      panelContents={panelContents}
      onToggle={handlePanelToggle}
    />
  );
};

export default BottomPanel;
