import { Position } from "@xyflow/react";
import { memo } from "react";

import { GeneralNodeProps } from "./GeneralNode";
import CustomHandle from "./GeneralNode/components/CustomHandle/CustomHandle";

type Props = GeneralNodeProps;

const EntranceNode: React.FC<Props> = ({ data }) => {
  return (
    <div className="rounded-l-sm rounded-r-3xl border border-node-entrance bg-node-entrance/40 px-4 py-6">
      <div>
        <CustomHandle
          id={data.inputs?.[0]}
          className="right-[10px] z-[1001] h-3/4 rounded-l-sm rounded-r-full px-2"
          type="source"
          position={Position.Right}
        />
      </div>
    </div>
  );
};

export default memo(EntranceNode);
