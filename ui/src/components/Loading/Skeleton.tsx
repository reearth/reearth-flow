import { cn } from "@flow/lib/utils";

import { FlowLogo } from "..";

import "./styles.css";
import { useT } from "@flow/lib/i18n";

const Loading: React.FC<{
  show?: boolean;
  className?: string;
  title?: string;
}> = ({ className }) => {
  const t = useT();
  return (
    <div className={cn("z-40 flex size-full justify-center", className)}>
      <div className="flex h-full items-center">
        <div className="flex flex-col gap-5">
          <div className="flex flex-col items-center gap-3">
            <FlowLogo
              className="loading-pulse"
              style={{ height: "80px", width: "80px" }}
            />
            <p className="font-thin">{t("Loading")}</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Loading;
