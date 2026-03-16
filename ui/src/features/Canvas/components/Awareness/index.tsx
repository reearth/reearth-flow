import { ViewportPortal } from "@xyflow/react";

import type { AwarenessUser } from "@flow/types";

import AwarenessEdge from "./AwarenessEdge";
import MultiCursor from "./MultiCursor";
import SelectionRectangle from "./SelectionRectangle";

type AwarenessProps = {
  users: Record<string, AwarenessUser>;
  currentWorkflowId: string;
};

const Awareness: React.FC<AwarenessProps> = ({ users, currentWorkflowId }) => {
  return (
    <ViewportPortal>
      {Object.entries(users).map(([key, user]) => {
        if (user.currentWorkflowId !== currentWorkflowId) return null;

        return (
          <div key={key}>
            {user.cursor && <MultiCursor user={user} />}
            {user.selectionRect && <SelectionRectangle user={user} />}
            {user.draggingEdge && <AwarenessEdge user={user} />}
          </div>
        );
      })}
    </ViewportPortal>
  );
};

export default Awareness;
