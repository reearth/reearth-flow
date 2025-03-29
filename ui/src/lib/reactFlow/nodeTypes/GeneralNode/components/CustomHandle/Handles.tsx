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
    <>
      {nodeType !== "reader" && inputs && inputs.length === 1 && (
        <CustomHandle
          id={inputs[0]}
          className="left-2 z-1001 w-[16px] rounded-l rounded-r-none"
          type="target"
          position={Position.Left}
        />
      )}
      {outputs && outputs.length === 1 && (
        <CustomHandle
          id={outputs[0]}
          className="right-2 z-1001 w-[16px] rounded-l-none rounded-r"
          type="source"
          position={Position.Right}
        />
      )}
      <div className="absolute inset-x-0 mx-auto w-[95%] rounded-b-md bg-primary">
        <div className="relative">
          {inputs &&
            inputs.length > 1 &&
            inputs.map((input, index) => (
              <div
                key={input + index}
                className="relative border-b px-1.5 py-0.5">
                <CustomHandle
                  type="target"
                  className={`left-1 w-[8px] rounded-none transition-colors ${index === (!outputs && inputs && inputs.length - 1) ? "rounded-bl-sm" : undefined}`}
                  position={Position.Left}
                  id={input}
                  // isConnectable={1}
                />
                <p className="pl-1 text-[10px] dark:font-light">{input}</p>
              </div>
            ))}
          {outputs &&
            outputs.length > 1 &&
            outputs.map((output, index) => (
              <div
                key={output + index}
                className="relative border-b px-1.5 py-0.5 last-of-type:border-none">
                <CustomHandle
                  type="source"
                  className={`right-1 w-[8px] rounded-none transition-colors ${index === (outputs && outputs.length - 1) ? "rounded-br-sm" : undefined}`}
                  position={Position.Right}
                  id={output}
                />
                <p className="pr-1 text-end text-[10px] dark:font-light">
                  {output}
                </p>
              </div>
            ))}
        </div>
      </div>
    </>
  );
};

export default memo(Handles);
