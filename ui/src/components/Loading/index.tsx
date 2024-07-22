import { DoubleArrowRightIcon } from "@radix-ui/react-icons";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean }> = () => {
  return (
    <div className="absolute left-0 top-0 z-40 flex h-screen w-full justify-center bg-background-900">
      <div className="flex h-full items-center">
        <div className="flex flex-col gap-5">
          <div className="flex gap-3">
            <FlowLogo
              id="loading-svg"
              className="mb-8 rounded-lg bg-red-900/50 p-1 text-zinc-200"
              style={{ height: "110px", width: "110px" }}
            />
            <DoubleArrowRightIcon className="size-[110px] text-zinc-600" />
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
