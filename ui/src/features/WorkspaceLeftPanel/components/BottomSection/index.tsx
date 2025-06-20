import { config } from "@flow/config";
import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useT } from "@flow/lib/i18n";

import { WorkspaceSettings } from "./components";

type Props = {
  route?: RouteOption;
};

const BottomSection: React.FC<Props> = ({ route }) => {
  const { version } = config();
  const t = useT();
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col">
        <div className="flex flex-1 flex-col gap-4 p-4">
          <WorkspaceSettings selected={route} />
        </div>
        <div>
          <div className="h-px bg-primary" />
          <div className="flex items-center px-2 py-1">
            <p className="text-xs font-thin text-muted-foreground select-none">
              {t("Version ")}
              {version ?? "X.X.X"}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export { BottomSection };
