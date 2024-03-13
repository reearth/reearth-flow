import { InputIcon, PlusCircledIcon } from "@radix-ui/react-icons";

import { CollapsiblePanel, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";

type Props = {};

const RightPanel: React.FC<Props> = () => {
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(false);

  const panelContents: PanelContent[] = [
    {
      id: "field-editor",
      title: "Field Editor",
      icon: <InputIcon />,
      component: (
        <div>
          <p>Some field</p>
          <input name="some input" placeholder="Some value should probably go here" />
        </div>
      ),
    },
    {
      id: "etc",
      icon: <PlusCircledIcon />,
      title: "More can go here",
      component: (
        <div>
          <p className="text-xs">MOREEeeeee</p>
        </div>
      ),
    },
  ];
  return (
    <CollapsiblePanel
      className="bg-zinc-800 ml-1 mb-1 mr-1 rounded-md"
      isOpen={!!isPanelOpen}
      togglePosition="end-left"
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default RightPanel;
