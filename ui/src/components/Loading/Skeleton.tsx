import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

import { FlowLogo } from "..";

import "./styles.css";

const Loading: React.FC<{
  show?: boolean;
  className?: string;
  title?: string;
  /** 0-100. Renders a determinate progress bar when provided. */
  progress?: number;
}> = ({ title, className, progress }) => {
  const t = useT();
  const hasProgress = typeof progress === "number";
  const clampedProgress = hasProgress
    ? Math.min(100, Math.max(0, progress))
    : 0;
  return (
    <div className={cn("z-40 flex size-full justify-center", className)}>
      <div className="flex h-full items-center">
        <div className="flex flex-col gap-5">
          <div className="flex flex-col items-center gap-3">
            <FlowLogo
              className="loading-pulse"
              style={{ height: "80px", width: "80px" }}
            />
            <p className="font-thin">{title || t("Loading")}</p>
            {hasProgress && (
              <div
                className="bg-secondary h-1.5 w-56 overflow-hidden rounded-full"
                role="progressbar"
                aria-valuenow={clampedProgress}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label={title || t("Loading")}>
                <div
                  className="bg-primary h-full rounded-full transition-[width] duration-200 ease-out"
                  style={{ width: `${clampedProgress}%` }}
                />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Loading;
