import { AwarenessUser } from "@flow/types";

type SelectionRectangleProps = {
  user: AwarenessUser;
};

const getRect = (r: {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
}) => ({
  x: Math.min(r.startX, r.currentX),
  y: Math.min(r.startY, r.currentY),
  width: Math.abs(r.currentX - r.startX),
  height: Math.abs(r.currentY - r.startY),
});

const SelectionRectangle: React.FC<SelectionRectangleProps> = ({ user }) => {
  const rect = user.selectionRect ? getRect(user.selectionRect) : null;
  if (!rect) return null;
  return (
    <div
      style={{
        position: "absolute",
        left: rect.x,
        top: rect.y,
        width: rect.width,
        height: rect.height,
        pointerEvents: "none",
        zIndex: 1999,
        border: `1px solid ${user.color}`,
        background: `${user.color}22`,
      }}
    />
  );
};

export default SelectionRectangle;
