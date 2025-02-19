import { config } from "@flow/config";
import { cn } from "@flow/lib/utils";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean; className?: string }> = ({
  className,
}) => {
  const { brandName } = config();
  return (
    <div
      className={cn(
        "absolute left-0 top-0 z-40 flex h-screen w-full justify-center bg-secondary",
        className,
      )}>
      <div className="flex h-full items-center">
        <div className="flex flex-col gap-5">
          <div className="flex flex-col items-center gap-8">
            <FlowLogo
              className="loading-pulse"
              style={{ height: "110px", width: "110px" }}
            />
            <p className="text-2xl font-thin"> {brandName || "Flow"}</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export { Loading };
