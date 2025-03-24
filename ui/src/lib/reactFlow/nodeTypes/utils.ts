import { NodeStatus } from "@flow/types";

export const getPropsFrom = (status?: NodeStatus) => {
  const style =
    status === "completed"
      ? "border-success"
      : status === "failed"
        ? "border-destructive"
        : status === "processing"
          ? "active-node-status-border"
          : status === "pending"
            ? "queued-node-status"
            : "";

  return {
    style,
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
