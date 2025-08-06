import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type {
  EngineReadyWorkflow,
  ProjectVariable,
  Workflow,
} from "@flow/types";

import { generateUUID } from "../generateUUID";

import { createSubGraphs } from "./createSubGraphs";

export const consolidateWorkflows = (
  name: string,
  projectVariables: ProjectVariable[] = [],
  workflows: Workflow[],
): EngineReadyWorkflow | undefined => {
  const defaultEntryWorkflow = workflows.find(
    (wf) => wf.id === DEFAULT_ENTRY_GRAPH_ID,
  );
  if (!defaultEntryWorkflow) return undefined;

  const newEntryId = generateUUID();

  const withVariables = Object.fromEntries(
    projectVariables.map((v) => [v.name, v.defaultValue]),
  );

  const convertedWorkflows = workflows.map((wf) => {
    return wf.id === DEFAULT_ENTRY_GRAPH_ID ? { ...wf, id: newEntryId } : wf;
  });

  const subGraphs = createSubGraphs(convertedWorkflows);

  const consolidatedWorkflow = {
    id: generateUUID(),
    name,
    entryGraphId: newEntryId,
    with: withVariables,
    graphs: subGraphs,
  };

  return consolidatedWorkflow;
};
