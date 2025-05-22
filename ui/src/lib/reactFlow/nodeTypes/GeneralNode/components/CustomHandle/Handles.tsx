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
    <div className="flex justify-between">
      {nodeType !== "reader" && inputs && (
        <div className="inset-x-0 mx-auto flex-1">
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
              <div className="flex items-center translate-x-0.5">
                <div className="size-1.5 bg-gray-300 rounded-full" />
                <p className="pl-1 text-[10px] dark:font-thin italic">
                  {input}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
      {outputs && (
        <div className="inset-x-0 mx-auto flex-1">
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
              <div className="flex items-center -translate-x-0.5">
                <p className="pr-1 text-end text-[10px] dark:font-thin italic">
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
