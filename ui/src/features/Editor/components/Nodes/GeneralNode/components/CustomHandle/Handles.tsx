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
          className="rounded-l rounded-r-none left-2 z-[1001] w-[16px]"
          type="target"
          position={Position.Left}
        />
      )}
      {outputs && outputs.length === 1 && (
        <CustomHandle
          id={outputs[0]}
          className="rounded-r rounded-l-none right-2 z-[1001] w-[16px]"
          type="source"
          position={Position.Right}
        />
      )}
      <div
        id="handle-wrapper"
        className="absolute bg-zinc-900 text-zinc-400 rounded-b-md ml-auto mr-auto left-0 right-0 w-[95%]">
        <div className="relative">
          {inputs &&
            inputs.length > 1 &&
            inputs.map((input, index) => (
              <div key={input + index} className="relative border-b border-zinc-800 py-0.5 px-1.5">
                <CustomHandle
                  type="target"
                  className={`left-1 w-[8px] rounded-none transition-colors ${index === (!outputs && inputs && inputs.length - 1) ? "rounded-bl-sm" : undefined}`}
                  position={Position.Left}
                  id={input}
                  // isConnectable={1}
                />
                <p className="text-[10px] font-light pl-1">{input}</p>
              </div>
            ))}
          {outputs &&
            outputs.length > 1 &&
            outputs.map((output, index) => (
              <div
                key={output + index}
                className="relative border-b border-zinc-800 py-0.5 px-1.5 last-of-type:border-none">
                <CustomHandle
                  type="source"
                  className={`right-1 w-[8px] rounded-none transition-colors ${index === (outputs && outputs.length - 1) ? "rounded-br-sm" : undefined}`}
                  position={Position.Right}
                  id={output}
                />
                <p className="text-[10px] font-light pr-1 text-end">{output}</p>
              </div>
            ))}
        </div>
      </div>
    </>
  );
};

export default memo(Handles);
