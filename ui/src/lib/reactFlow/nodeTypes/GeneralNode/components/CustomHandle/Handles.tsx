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
        <div className="inset-x-0 mx-auto flex-1 min-w-0">
          {inputs.map((input, index) => (
            <div
              key={input + index}
              className="relative border-b py-0.5 flex items-center last-of-type:border-none">
              <CustomHandle
                type="target"
                className={`left-1 w-[8px] rounded-none transition-colors ${index === (!outputs && inputs && inputs.length - 1) ? "rounded-bl-sm" : undefined}`}
                position={Position.Left}
                id={input}
                // isConnectable={1}
              />
              <div className="flex items-center translate-x-0.5 w-full">
                <div>
                  <div className="size-1.5 bg-gray-300 rounded-full" />
                </div>
                <p className="pl-1 text-[10px] dark:font-thin italic break-words w-[90%]">
                  {input}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
      {outputs && (
        <div className="inset-x-0 mx-auto flex-1 min-w-0 overflow-hidden">
          {outputs.map((output, index) => (
            <div
              key={output + index}
              className="relative flex justify-end items-center border-b py-0.5 last-of-type:border-none">
              <CustomHandle
                type="source"
                className="w-[8px] right-1 rounded-none transition-colors z-10"
                position={Position.Right}
                id={output}
              />
              <div className="flex justify-end items-center -translate-x-0.5 w-full">
                <p className="pr-1 text-end text-[10px] dark:font-thin italic break-words w-[90%]">
                  {output}
                </p>
                <div className="size-1.5 bg-gray-300 rounded-full" />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default memo(Handles);
