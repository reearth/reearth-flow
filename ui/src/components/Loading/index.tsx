import { DoubleArrowRightIcon } from "@radix-ui/react-icons";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean }> = ({ show }) => {
  return (
    show && (
      <div className="absolute top-0 z-40 flex flex-col items-center pt-64 h-[100vh] w-full bg-zinc-900">
        <div className="flex gap-3">
          <FlowLogo
            id="loading-svg"
            className="bg-red-900 rounded-lg p-1 mb-8"
            style={{ height: "130px", width: "130px" }}
          />
          <DoubleArrowRightIcon className="w-[130px] h-[130px] text-zinc-600" />
        </div>
        {/* <p id="loading-text" className="text-2xl text-zinc-500">
        Loading...
      </p> */}
      </div>
    )
  );
};

export { Loading };
