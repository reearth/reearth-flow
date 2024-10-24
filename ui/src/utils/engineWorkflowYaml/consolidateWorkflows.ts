import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Workflow } from "@flow/types";

import { randomID } from "../randomID";

import { createSubGraphs } from "./createSubGraphs";

export const consolidateWorkflows = (name: string, workflows: Workflow[]) => {
  const entryGraphId =
    workflows.find((wf) => wf.id === DEFAULT_ENTRY_GRAPH_ID)?.id ??
    DEFAULT_ENTRY_GRAPH_ID;
  const subGraphs = createSubGraphs(workflows);

  const consolidatedWorkflow = {
    id: randomID(),
    name,
    entryGraphId,
    // with // TODO: conversion of data.params to with
    graphs: subGraphs,
  };

  return consolidatedWorkflow;
};
