import { Algorithm, EngineReadyWorkflow, Workflow } from "@flow/types";

import { separateWorkflow } from "./separateWorkflows";

type Meta = {
  name: string;
};

export type WorkflowVariable = {
  name: string;
  value: any;
};

export const deconstructedEngineWorkflow = async ({
  engineWorkflow,
  layoutType,
}: {
  engineWorkflow?: EngineReadyWorkflow;
  layoutType?: Algorithm;
}): Promise<
  | { meta: Meta; workflows: Workflow[]; variables?: WorkflowVariable[] }
  | undefined
> => {
  if (!engineWorkflow) return;
  const meta = { name: engineWorkflow.name };

  const canvasReadyWorkflows: Workflow[] | undefined = await separateWorkflow({
    engineWorkflow,
    layoutType,
  });

  if (!canvasReadyWorkflows) return;

  // Extract workflow variables from the 'with' field
  const variables: WorkflowVariable[] = engineWorkflow.with
    ? Object.entries(engineWorkflow.with).map(([name, value]) => ({
        name,
        value,
      }))
    : [];

  return {
    meta,
    workflows: canvasReadyWorkflows,
    variables: variables.length > 0 ? variables : undefined,
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
