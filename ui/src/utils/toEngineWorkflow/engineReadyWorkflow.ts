import { EngineReadyWorkflow, Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";

export const createEngineReadyWorkflow = (
  name?: string,
  workflows?: Workflow[],
): EngineReadyWorkflow | undefined => {
  if (!workflows) return;
  const engineReadyWorkflow: EngineReadyWorkflow | undefined =
    consolidateWorkflows(`${name ?? "Untitled"}-workflow`, workflows);

  if (!engineReadyWorkflow) return;

  return engineReadyWorkflow;
};
