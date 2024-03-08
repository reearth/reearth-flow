import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean }> = ({ show }) => {
  return (
    show && (
      <div className="absolute top-0 z-40 flex flex-col items-center pt-64 h-[100vh] w-full bg-zinc-800">
        <FlowLogo
          id="loading-svg"
          className="bg-red-900 rounded-lg p-1 mb-8"
          style={{ height: "150px", width: "150px" }}
        />
        {/* <p id="loading-text" className="text-2xl text-zinc-500">
        Loading...
      </p> */}
      </div>
    )
  );
};

export { Loading };
