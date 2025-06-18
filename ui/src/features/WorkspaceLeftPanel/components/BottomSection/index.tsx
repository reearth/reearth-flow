import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";

import { WorkspaceSettings } from "./components";

type Props = {
  route?: RouteOption;
};

const BottomSection: React.FC<Props> = ({ route }) => {
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col">
        <div className="flex flex-1 flex-col gap-4 p-4">
          <WorkspaceSettings selected={route} />
        </div>
      </div>
    </div>
  );
};

export { BottomSection };
