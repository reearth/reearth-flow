import { CollapsiblePanel, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";

export type BottomPanelProps = {};

const BottomPanel: React.FC<BottomPanelProps> = () => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);

  const panelContents: PanelContent[] = [
    {
      id: "translation-log",
      component: (
        // <div className="flex justify-between h-full" style={{ color: "#dbdbdb" }}>
        //   <div className="flex gap-6 ml-[18px]">
        <p className="text-sm">Translation Log</p>
        //   <p className="text-sm">Visual Preview</p>
        // </div>
        // {/* <div className="self-end">
        //   <p className="text-xs">yokohama_river.proj - CDED -&#62; NONE - Flow 2024 &#169;</p>
        // </div> */}
        // </div>
      ),
    },
    {
      id: "visual-preview",
      component: <p className="text-sm">Visual Preview</p>,
    },
  ];

  return (
    <CollapsiblePanel
      className="bg-zinc-950 mr-1 mb-1 rounded-md"
      direction="horizontal"
      isOpen={!!isPanelOpen}
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default BottomPanel;
