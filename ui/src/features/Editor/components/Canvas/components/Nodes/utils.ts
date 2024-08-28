import { Status } from "@flow/types";

export const getPropsFrom = (status?: Status) => {
  const style =
    status === "success"
      ? "bg-success"
      : status === "failure"
        ? "bg-destructive"
        : status === "active"
          ? "active-node-status"
          : "bg-primary";

  const isAnimated = status === "active";
  return {
    style,
    animated: isAnimated,
  };
};
