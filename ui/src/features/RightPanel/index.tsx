import { Node } from "@flow/types";

import { ParamEditor } from "../Editor/components/ParamEditor";

type Props = {
  selected?: Node;
};

const RightPanel: React.FC<Props> = ({ selected }) => {
  return (
    <div
      id="right-panel"
      className="bg-zinc-800 border-l border-zinc-700 fixed right-0 h-full w-[350px] transition-all"
      style={{
        transform: `translateX(${selected ? "0" : "100%"})`,
        transitionDuration: selected ? "500ms" : "300ms",
        transitionProperty: "transform",
        transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
      }}>
      <div className="bg-zinc-900/50 w-full h-full py-4 pl-4 pr-2">
        {selected && (
          <ParamEditor nodeId={selected.id} nodeMeta={selected.data} nodeType="transformer" />
        )}
      </div>
    </div>
  );
};

export default RightPanel;
