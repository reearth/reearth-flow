import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { EngineReadyWorkflow, Workflow } from "@flow/types";

import { generateUUID } from "../generateUUID";

import { createSubGraphs } from "./createSubGraphs";

export const consolidateWorkflows = (
  name: string,
  workflows: Workflow[],
): EngineReadyWorkflow | undefined => {
  const entryGraphId = workflows.find(
    (wf) => wf.id === DEFAULT_ENTRY_GRAPH_ID,
  )?.id;
  if (!entryGraphId) return undefined;

  const convertedWorkflows = workflows.map((wf) => {
    return wf.id === DEFAULT_ENTRY_GRAPH_ID ? { ...wf, id: entryGraphId } : wf;
  });

  const subGraphs = createSubGraphs(convertedWorkflows);

  const consolidatedWorkflow = {
    id: generateUUID(),
    name,
    entryGraphId: generateUUID(),
    // with // TODO: conversion of data.params to with
    graphs: subGraphs,
  };

  return consolidatedWorkflow;
};
