import YAML from "yaml";

import { Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";

export const createWorkflowsYaml = (name?: string, workflows?: Workflow[]) => {
  if (!workflows) return;
  const yamlReadyWorkflow = consolidateWorkflows(
    `${name ?? "Untitled"}-workflow`,
    workflows,
  );

  const yamlWorkflow = YAML.stringify(yamlReadyWorkflow);

  return { workflowId: yamlReadyWorkflow.id, yamlWorkflow };
};
