import { Position } from "@xyflow/react";
import { memo } from "react";

import { GeneralNodeProps } from "./GeneralNode";
import CustomHandle from "./GeneralNode/components/CustomHandle/CustomHandle";

type Props = GeneralNodeProps;

const EntranceNode: React.FC<Props> = ({ data }) => {
  return (
    <div className="rounded-l-sm rounded-r-3xl border border-[#a21caf] bg-[#a21caf]/40 px-3 py-4">
      {/* <Intersection className="rotate-90" weight="thin" /> */}
      <CustomHandle
        id={data.inputs?.[0]}
        className="right-3 z-[1001] rounded-l-sm rounded-r-3xl px-3 py-4"
        type="source"
        position={Position.Right}
      />
    </div>
  );
};

export default memo(EntranceNode);
