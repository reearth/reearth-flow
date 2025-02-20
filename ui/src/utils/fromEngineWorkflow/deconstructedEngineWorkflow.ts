import { EngineReadyWorkflow, Workflow } from "@flow/types";

import { separateWorkflow } from "./separateWorkflows";

type Meta = {
  name: string;
};

export const deconstructedEngineWorkflow = async (
  engineWorkflow?: EngineReadyWorkflow,
): Promise<{ meta: Meta; workflows: Workflow[] } | undefined> => {
  if (!engineWorkflow) return;
  const meta = { name: engineWorkflow.name };

  const canvasReadyWorkflows: Workflow[] | undefined =
    await separateWorkflow(engineWorkflow);

  if (!canvasReadyWorkflows) return;

  return {
    meta,
    workflows: canvasReadyWorkflows,
  };
};

export const isEngineWorkflow = (workflow: any): boolean => {
  return (
    typeof workflow === "object" &&
    workflow !== null &&
    "name" in workflow &&
    "graphs" in workflow
  );
};
