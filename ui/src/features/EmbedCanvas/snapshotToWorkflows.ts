import * as Y from "yjs";

import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Workflow } from "@flow/types";

export const snapshotToWorkflows = (updates: number[]): Workflow[] => {
  const doc = new Y.Doc();
  try {
    Y.applyUpdate(doc, new Uint8Array(updates));
    const yWorkflows = doc.getMap<YWorkflow>("workflows");
    return Array.from(yWorkflows.values()).map((yWorkflow) =>
      rebuildWorkflow(yWorkflow),
    );
  } finally {
    doc.destroy();
  }
};
