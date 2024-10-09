import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean }> = () => {
  return (
    <div className="absolute left-0 top-0 z-40 flex h-screen w-full justify-center bg-secondary">
      <div className="flex h-full items-center">
        <div className="flex flex-col gap-5">
          <div className="flex flex-col items-center gap-8">
            <FlowLogo
              id="loading-svg"
              style={{ height: "110px", width: "110px" }}
            />
            <p className="text-2xl font-thin">Re:Earth Flow</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export { Loading };
