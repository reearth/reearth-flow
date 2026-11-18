import type * as Y from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { YDocMetadataValue } from "@flow/types";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

/**
 * Gives a project that has never been opened its Main Workflow.
 *
 * This is the one document write that happens without a user action, and only
 * once per project. Writes nothing when the document is already initialised,
 * or when it predates the flag but already has a Main Workflow.
 */
export const initializeProjectDoc = (yDoc: Y.Doc) => {
  const metadata = yDoc.getMap<YDocMetadataValue>("metadata");
  if (metadata.get("initialized")) return;

  yDoc.transact(() => {
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    // Older projects have a Main Workflow but no flag.
    if (yWorkflows.get(DEFAULT_ENTRY_GRAPH_ID)) return;
    // Only one client should set this flag.
    if (metadata.get("initialized")) return;
    yWorkflows.set(
      DEFAULT_ENTRY_GRAPH_ID,
      yWorkflowConstructor(DEFAULT_ENTRY_GRAPH_ID, "Main Workflow"),
    );
    metadata.set("initialized", true);
  });
};
