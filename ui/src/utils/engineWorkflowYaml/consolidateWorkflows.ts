import type { Workflow } from "@flow/types";

import { randomID } from "../randomID";

import { createSubGraphs } from "./createSubGraphs";

export const consolidateWorkflows = (name: string, workflows: Workflow[]) => {
  const entryGraphId = workflows.find((wf) => wf.id === "main")?.id;
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
