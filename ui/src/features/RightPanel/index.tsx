import { Node } from "@flow/types";

import { ParamEditor } from "../Editor/components/ParamEditor";

type Props = {
  selected?: Node[];
};

const RightPanel: React.FC<Props> = ({ selected }) => {
  const node = selected?.[0];

  return (
    <div
      id="right-panel"
      className="bg-zinc-800 border-l border-zinc-700 fixed right-0 h-full w-[350px] transition-all"
      style={{
        transform: `translateX(${node ? "0" : "100%"})`,
        transitionDuration: node ? "500ms" : "300ms",
        transitionProperty: "transform",
        transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
      }}>
      <div className="bg-zinc-900/50 w-full h-full py-4 pl-4 pr-2">
        {node && <ParamEditor nodeId={node.id} nodeMeta={node.data} nodeType="transformer" />}
      </div>
    </div>
  );
};

export default RightPanel;
