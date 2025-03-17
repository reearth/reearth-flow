import { createPortal } from "react-dom";

import { config } from "@flow/config";
import { cn } from "@flow/lib/utils";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{ show?: boolean; className?: string }> = ({
  show = true,
  className,
}) => {
  const { brandName } = config();

  // Don't render anything if show is false
  if (!show) return null;

  // Portal content to render at document.body level
  const loadingContent = (
    <div
      className={cn(
        "fixed left-0 top-0 z-50 flex h-screen w-screen justify-center items-center bg-secondary",
        className,
      )}>
      <div className="flex flex-col items-center gap-8">
        <FlowLogo
          className="loading-pulse"
          style={{ height: "110px", width: "110px" }}
        />
        <p className="text-2xl font-thin">{brandName || "Flow"}</p>
      </div>
    </div>
  );

  // Use portal to render at the root level, escaping any parent containers
  return createPortal(loadingContent, document.body);
};

export default Loading;
