import type { AwarenessUser } from "@flow/types";

import { Cursor } from "./PerfectCursor";

type MultiCursorProps = {
  user: AwarenessUser;
};

const MultiCursor: React.FC<MultiCursorProps> = ({ user }) => {
  return (
    <div
      className="pointer-events-none absolute"
      style={{
        left: user?.cursor?.x,
        top: user?.cursor?.y,
        zIndex: 2000,
      }}>
      <Cursor color={user.color} point={[0, 0]} userName={user.userName} />
    </div>
  );
};

export default MultiCursor;
