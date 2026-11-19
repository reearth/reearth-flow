import { describe, expect, it } from "vitest";

import type { JobState } from "@flow/stores";
import type { AnyWorkflowVariable, Edge, Node, Workflow } from "@flow/types";

import { pickIncrementalStart } from "./pickIncrementalStart";

const node = (id: string, type: Node["type"] = "transformer"): Node => ({
  id,
  type,
  position: { x: 0, y: 0 },
  data: { officialName: id },
});

const edge = (source: string, target: string, targetHandle?: string): Edge => ({
  id: `${source}->${target}`,
  source,
  target,
  targetHandle,
});

const chain: Workflow[] = [
  {
    id: "main",
    nodes: [node("R", "reader"), node("A"), node("B"), node("C")],
    edges: [edge("R", "A"), edge("A", "B"), edge("B", "C")],
  },
];

const variable = (name: string, defaultValue: unknown) =>
  ({
    id: name,
    name,
    defaultValue,
    type: "text",
    required: false,
    public: false,
  }) as AnyWorkflowVariable;

const variables = [variable("city", "tsukuba")];

const lastRun = (overrides: Partial<JobState> = {}): JobState => ({
  projectId: "p",
  jobId: "job-1",
  status: "completed",
  variables,
  ranNodeIds: ["R", "A", "B", "C"],
  graphSnapshot: {
    nodeHashes: {},
    nodeIds: {},
    edgeSignatures: {},
    subworkflowBridges: [],
  },
  isRunStale: false,
  staleNodeIds: [],
  ...overrides,
});

const pick = (
  run: JobState | undefined,
  workflows: Workflow[] = chain,
  selectedNodeId = "C",
  vars: AnyWorkflowVariable[] = variables,
) =>
  pickIncrementalStart({
    trimmedWorkflows: workflows,
    selectedNodeId,
    lastRun: run,
    variables: vars,
  });

describe("pickIncrementalStart", () => {
  it("re-runs only the selected action when nothing changed", () => {
    expect(pick(lastRun())).toEqual({
      previousJobId: "job-1",
      startNodeId: "C",
    });
  });

  it("starts at the one action that changed", () => {
    const run = lastRun({ isRunStale: true, staleNodeIds: ["A", "B", "C"] });
    expect(pick(run)).toEqual({ previousJobId: "job-1", startNodeId: "A" });
  });

  it("ignores changes after the selected action", () => {
    const run = lastRun({ isRunStale: true, staleNodeIds: ["D", "E"] });
    expect(pick(run)).toEqual({ previousJobId: "job-1", startNodeId: "C" });
  });

  it("ignores stale ids left over once the run is no longer stale", () => {
    const run = lastRun({ isRunStale: false, staleNodeIds: ["A"] });
    expect(pick(run)?.startNodeId).toBe("C");
  });

  it("runs fresh when two separate actions changed", () => {
    const workflows: Workflow[] = [
      {
        id: "main",
        nodes: [
          node("R1", "reader"),
          node("R2", "reader"),
          node("A"),
          node("B"),
          node("J"),
        ],
        edges: [
          edge("R1", "A"),
          edge("R2", "B"),
          edge("A", "J", "left"),
          edge("B", "J", "right"),
        ],
      },
    ];
    const run = lastRun({
      isRunStale: true,
      staleNodeIds: ["A", "B", "J"],
      ranNodeIds: ["R1", "R2", "A", "B", "J"],
    });

    expect(pick(run, workflows, "J")).toBeUndefined();
  });

  it("runs fresh when the last run did not produce what would be reused", () => {
    const run = lastRun({
      isRunStale: true,
      staleNodeIds: ["B", "C"],
      ranNodeIds: ["B", "C"],
    });
    expect(pick(run)).toBeUndefined();
  });

  it("runs fresh when the variables changed", () => {
    expect(
      pick(lastRun(), chain, "C", [variable("city", "kofu")]),
    ).toBeUndefined();
  });

  it("runs fresh without a usable last run", () => {
    expect(pick(undefined)).toBeUndefined();
    expect(pick(lastRun({ status: "running" }))).toBeUndefined();
    expect(pick(lastRun({ joinedUserName: "someone" }))).toBeUndefined();
    expect(pick(lastRun({ ranNodeIds: undefined }))).toBeUndefined();
    expect(pick(lastRun({ graphSnapshot: undefined }))).toBeUndefined();
  });

  it("runs fresh when the start would be a subworkflow node", () => {
    const workflows: Workflow[] = [
      {
        id: "main",
        nodes: [
          node("R", "reader"),
          {
            ...node("S", "subworkflow"),
            data: {
              officialName: "S",
              subworkflowId: "S",
              pseudoInputs: [{ nodeId: "IR", portName: "in" }],
              pseudoOutputs: [{ nodeId: "OR", portName: "out" }],
            },
          },
        ],
        edges: [edge("R", "S", "in")],
      },
      {
        id: "S",
        nodes: [node("IR"), node("OR")],
        edges: [edge("IR", "OR")],
      },
    ];
    const run = lastRun({ ranNodeIds: ["R", "S", "IR", "OR"] });

    expect(pick(run, workflows, "S")).toBeUndefined();
  });
});
