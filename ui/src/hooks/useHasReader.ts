import { useMemo } from "react";

import { hasReader } from "@flow/lib/fetch";
import type { Node } from "@flow/types";

export default (nodes: Node[]) => {
  return useMemo(() => hasReader(nodes), [nodes]);
};
