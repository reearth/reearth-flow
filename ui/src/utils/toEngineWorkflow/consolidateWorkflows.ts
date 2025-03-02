import type { EngineReadyWorkflow, Workflow } from "@flow/types";

import { generateUUID } from "../generateUUID";

import { createSubGraphs } from "./createSubGraphs";

export const consolidateWorkflows = (
  name: string,
  workflows: Workflow[],
): EngineReadyWorkflow | undefined => {
  const defaultEntryWorkflow = workflows.find((wf) => wf.isMain);
  if (!defaultEntryWorkflow) return undefined;

  const subGraphs = createSubGraphs(workflows);

  const consolidatedWorkflow = {
    id: generateUUID(),
    name,
    entryGraphId: defaultEntryWorkflow.id,
    // with // TODO: conversion of data.params to with
    graphs: subGraphs,
  };

  return consolidatedWorkflow;
};
