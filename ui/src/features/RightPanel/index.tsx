// import { Button } from "@flow/components";
import { Node } from "@flow/types";

import { ParamEditor } from "../Canvas/components/ParamEditor";

type Props = {
  selected?: Node[];
};

const RightPanel: React.FC<Props> = ({ selected }) => {
  const node = selected?.[0];

  return (
    <div
      id="right-panel"
      className="bg-zinc-800 border-l border-zinc-700 py-4 pl-4 pr-2 absolute right-0 h-full w-[400px]"
      style={{
        transform: `translateX(${node ? "0" : "100%"})`,
        transitionDuration: node ? "500ms" : "300ms",
        transitionProperty: "transform",
        transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
      }}>
      {node && <ParamEditor nodeId={node.id} nodeMeta={node.data} nodeType="transformer" />}
    </div>
  );
};

export default RightPanel;
