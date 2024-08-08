import { Status } from "@flow/types";

export const getPropsFrom = (status?: Status) => {
  const style =
    status === "success"
      ? "bg-green-500"
      : status === "failure"
        ? "bg-red-500"
        : status === "active"
          ? "active-node-status"
          : "bg-primary";

  const isAnimated = status === "active";
  return {
    style,
    animated: isAnimated,
  };
};
