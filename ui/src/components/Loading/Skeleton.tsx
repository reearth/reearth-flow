import { cn } from "@flow/lib/utils";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean; className?: string }> = ({
  className,
}) => {
  return (
    <div className={cn("z-40 flex size-full justify-center", className)}>
      <div className="flex h-full items-center">
        <div className="flex flex-col gap-5">
          <div className="flex flex-col items-center gap-8">
            <FlowLogo
              className="loading-pulse"
              style={{ height: "80px", width: "80px" }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default Loading;
