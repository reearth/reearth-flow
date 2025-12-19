import { EngineReadyWorkflow, WorkflowVariable, Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";

export const createEngineReadyWorkflow = (
  name?: string,
  workflowVariables?: WorkflowVariable[],
  workflows?: Workflow[],
): EngineReadyWorkflow | undefined => {
  if (!workflows) return;
  const engineReadyWorkflow: EngineReadyWorkflow | undefined =
    consolidateWorkflows(
      `${name ?? "Untitled"}-workflow`,
      workflowVariables,
      workflows,
    );

  if (!engineReadyWorkflow) return;

  return engineReadyWorkflow;
};
