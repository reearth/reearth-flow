import { Algorithm, EngineReadyWorkflow, Workflow } from "@flow/types";

import { separateWorkflow } from "./separateWorkflows";

type Meta = {
  name: string;
};

export const deconstructedEngineWorkflow = async ({
  engineWorkflow,
  layoutType,
}: {
  engineWorkflow?: EngineReadyWorkflow;
  layoutType?: Algorithm;
}): Promise<{ meta: Meta; workflows: Workflow[] } | undefined> => {
  console.log("HERE w engineWorkflow", engineWorkflow);
  if (!engineWorkflow) return;
  const meta = { name: engineWorkflow.name };

  console.log("HERE");
  const canvasReadyWorkflows: Workflow[] | undefined = await separateWorkflow({
    engineWorkflow,
    layoutType,
  });

  console.log("canvasReadyWorkflows", canvasReadyWorkflows);

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
