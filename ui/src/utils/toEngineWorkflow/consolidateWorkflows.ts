import type { EngineReadyWorkflow, Workflow } from "@flow/types";

import { generateUUID } from "../generateUUID";

import { createSubGraphs } from "./createSubGraphs";

export const consolidateWorkflows = (
  name: string,
  workflows: Workflow[],
): EngineReadyWorkflow | undefined => {
  const mainWorkflowId = workflows.find((wf) => wf.isMain)?.id;
  if (!mainWorkflowId) return undefined;

  const subGraphs = createSubGraphs(workflows);

  const consolidatedWorkflow = {
    id: generateUUID(),
    name,
    entryGraphId: mainWorkflowId,
    // with // TODO: conversion of data.params to with
    graphs: subGraphs,
  };

  return consolidatedWorkflow;
};
