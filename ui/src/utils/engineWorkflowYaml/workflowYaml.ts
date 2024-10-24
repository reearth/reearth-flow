import YAML from "yaml";

import { EngineReadyWorkflow, Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";

export const createWorkflowsYaml = (name?: string, workflows?: Workflow[]) => {
  if (!workflows) return;
  const yamlReadyWorkflow: EngineReadyWorkflow | undefined =
    consolidateWorkflows(`${name ?? "Untitled"}-workflow`, workflows);

  if (!yamlReadyWorkflow) return;

  const yamlWorkflow = YAML.stringify(yamlReadyWorkflow);

  return { workflowId: yamlReadyWorkflow.id, yamlWorkflow };
};
