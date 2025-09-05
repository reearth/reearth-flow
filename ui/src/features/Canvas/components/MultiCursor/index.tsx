import { ViewportPortal } from "@xyflow/react";
import type { Awareness } from "y-protocols/awareness";

import { Cursor } from "./PerfectCursor";

type MultiCursorProps = {
  users: any;
  yAwareness: Awareness;
  currentUserName?: string;
};

type UserData = {
  color?: string;
  cursor?: { x: number; y: number };
};

const MultiCursor: React.FC<MultiCursorProps> = ({ users, yAwareness }) => {
  return (
    <ViewportPortal>
      {Array.from(users.entries() as IterableIterator<[number, UserData]>).map(
        ([key, value]) => {
          if (key === yAwareness.clientID) return null;
          if (!value.cursor) return null;

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
              <Cursor color={value.color} point={[0, 0]} />
            </div>
          );
        },
      )}
    </ViewportPortal>
  );
};

export default MultiCursor;
