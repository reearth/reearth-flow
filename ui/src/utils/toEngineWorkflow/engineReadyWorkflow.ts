import { EngineReadyWorkflow, ProjectVariable, Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";

export const createEngineReadyWorkflow = (
  name?: string,
  projectVariables?: ProjectVariable[],
  workflows?: Workflow[],
): EngineReadyWorkflow | undefined => {
  if (!workflows) return;
  const engineReadyWorkflow: EngineReadyWorkflow | undefined =
    consolidateWorkflows(
      `${name ?? "Untitled"}-workflow`,
      projectVariables,
      workflows,
    );

  if (!engineReadyWorkflow) return;

  return engineReadyWorkflow;
};
