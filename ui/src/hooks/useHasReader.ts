import { useMemo } from "react";

import { type Node } from "@flow/types";

export default (nodes: Node[]) => {
  return useMemo(() => {
    return nodes?.some((node) => node.type === "reader");
  }, [nodes]);
};
