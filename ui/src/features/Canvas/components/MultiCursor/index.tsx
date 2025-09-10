import { ViewportPortal } from "@xyflow/react";

import type { AwarenessUser } from "@flow/types";

import { Cursor } from "./PerfectCursor";

type MultiCursorProps = {
  users: Record<string, AwarenessUser>;
  currentWorkflowId: string;
};

const MultiCursor: React.FC<MultiCursorProps> = ({ users, currentWorkflowId }) => {
  return (
    <ViewportPortal>
      {Object.entries(users).map(([key, value]) => {
        if (!value.cursor || value.currentWorkflowId !== currentWorkflowId) return null;

        return (
          <div
            key={key}
            style={{
              position: "absolute",
              left: value.cursor.x,
              top: value.cursor.y,
              pointerEvents: "none",
              zIndex: 1000,
            }}>
            <Cursor
              color={value.color}
              point={[0, 0]}
              userName={value.userName}
            />
          </div>
        );
      })}
    </ViewportPortal>
  );
};

export default MultiCursor;
