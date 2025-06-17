import { Position } from "@xyflow/react";
import { memo } from "react";

import CustomHandle from "./CustomHandle";

type Props = {
  nodeType?: string;
  inputs?: string[];
  outputs?: string[];
};

const Handles: React.FC<Props> = ({ nodeType, inputs, outputs }) => {
  return (
    <div className="flex justify-between gap-0.5">
      {nodeType !== "reader" && inputs && (
        <div className="inset-x-0 mx-auto min-w-0 flex-1">
          {inputs.map((input, index) => (
            <div
              key={input + index}
              className="relative flex items-center border-b py-0.5 last-of-type:border-none">
              <CustomHandle
                type="target"
                className={`left-1 w-[8px] rounded-none transition-colors ${index === (!outputs && inputs && inputs.length - 1) ? "rounded-bl-sm" : undefined}`}
                position={Position.Left}
                id={input}
                // isConnectable={1}
              />
              <div className="flex w-full translate-x-0.5 items-center">
                <div>
                  <div className="size-1.5 rounded-full bg-gray-300" />
                </div>
                <p className="w-[90%] pl-1 text-[10px] break-words italic dark:font-thin">
                  {input}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
      {outputs && (
        <div className="inset-x-0 mx-auto min-w-0 flex-1 overflow-hidden">
          {outputs.map((output, index) => (
            <div
              key={output + index}
              className="relative flex items-center justify-end border-b py-0.5 last-of-type:border-none">
              <CustomHandle
                type="source"
                className="right-1 z-10 w-[8px] rounded-none transition-colors"
                position={Position.Right}
                id={output}
              />
              <div className="flex w-full -translate-x-0.5 items-center justify-end">
                <p className="w-[90%] pr-1 text-end text-[10px] break-words italic dark:font-thin">
                  {output}
                </p>
                <div className="size-1.5 rounded-full bg-gray-300" />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default memo(Handles);
