import { DoubleArrowRightIcon } from "@radix-ui/react-icons";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean }> = () => {
  return (
    <div className="absolute top-0 z-40 flex justify-center h-[100vh] w-full bg-zinc-900">
      <div className="flex items-center h-full">
        <div className="flex flex-col gap-5">
          <div className="flex gap-3">
            <FlowLogo
              id="loading-svg"
              className="bg-red-900 bg-opacity-50 text-zinc-200 rounded-lg p-1 mb-8"
              style={{ height: "110px", width: "110px" }}
            />
            <DoubleArrowRightIcon className="w-[110px] h-[110px] text-zinc-600" />
          </div>
        </div>
      </div>
      {/* <p id="loading-text" className="text-2xl text-zinc-500">
        Loading...
      </p> */}
    </div>
  );
};

export { Loading };
