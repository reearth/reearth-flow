import { NodeStatus } from "@flow/types";

export const getPropsFrom = (status?: NodeStatus) => {
  const style =
    status === "succeeded"
      ? "bg-success"
      : status === "failed"
        ? "bg-destructive"
        : status === "running"
          ? "active-node-status"
          : status === "pending"
            ? "queued-node-status"
            : "bg-primary";

  const isAnimated = status === "running";
  return {
    style,
    animated: isAnimated,
  };
};

export const convertHextoRgba = (hex: string, alpha: number) => {
  let r, g, b;

  if (hex.length === 4) {
    hex = `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}`;
  }
  if (hex.length === 7) {
    r = parseInt(hex.slice(1, 3), 16);
    g = parseInt(hex.slice(3, 5), 16);
    b = parseInt(hex.slice(5, 7), 16);

    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  return hex;
};
