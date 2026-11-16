import { describe, expect, test } from "vitest";

import type {
  DiagnosticFragment,
  JobFragment,
  ProjectFragment,
} from "./__gen__/plugins/graphql-request";
import {
  toDiagnostic,
  toJob,
  toProject,
  toRunParameterInputs,
} from "./convert";

const diagnosticFragment = (
  overrides: Partial<DiagnosticFragment> = {},
): DiagnosticFragment => ({
  code: "geometry_invalid",
  category: "geometry",
  severity: "warn",
  effectiveDisposition: "warn_drop",
  nodeId: "node-1",
  actionType: "GeometryValidator",
  featureId: null,
  message: "self-intersecting polygon",
  help: null,
  aggregatedCount: null,
  sampleFeatureIds: null,
  ...overrides,
});

const jobFragment = (overrides: Partial<JobFragment> = {}): JobFragment => ({
  id: "job-1",
  workspaceId: "workspace-1",
  status: "COMPLETED",
  startedAt: "2024-01-25T09:15:00Z",
  completedAt: "2024-01-25T09:18:45Z",
  outputURLs: null,
  userFacingLogsURL: null,
  debug: false,
  deployment: null,
  runById: null,
  triggerId: null,
  trigger: null,
  ...overrides,
});

describe("toDiagnostic", () => {
  test("passes category, severity and disposition through verbatim", () => {
    // They are plain strings on the wire precisely so a value this build has
    // never seen survives; coercing them to something known would lose it.
    const converted = toDiagnostic(
      diagnosticFragment({
        category: "quantum",
        severity: "catastrophic",
        effectiveDisposition: "vaporise",
      }),
    );

    expect(converted.category).toBe("quantum");
    expect(converted.severity).toBe("catastrophic");
    expect(converted.effectiveDisposition).toBe("vaporise");
  });

  test("drops nulls to undefined", () => {
    const converted = toDiagnostic(diagnosticFragment());

    expect(converted.featureId).toBeUndefined();
    expect(converted.help).toBeUndefined();
    expect(converted.aggregatedCount).toBeUndefined();
    expect(converted.sampleFeatureIds).toBeUndefined();
  });

  test("keeps an aggregated count of zero distinct from an absent one", () => {
    expect(
      toDiagnostic(diagnosticFragment({ aggregatedCount: 0 })).aggregatedCount,
    ).toBe(0);
  });
});

describe("toJob", () => {
  test("carries no diagnostics payload", () => {
    // failedNodes and droppedEventCount each have their own per-job resolver on
    // the server. They were on the shared Job fragment, which made the
    // workspace jobs list resolve and transfer both for every row it rendered
    // without them. They are fetched with the diagnostics batch instead.
    const converted = toJob(jobFragment({ status: "FAILED" }));

    expect(converted).not.toHaveProperty("failedNodes");
    expect(converted).not.toHaveProperty("droppedEventCount");
  });

  test("leaves variables unset when the query did not select them", () => {
    expect(toJob(jobFragment()).variables).toBeUndefined();
  });

  test("types the variables a run used as the UI types them", () => {
    const converted = toJob({
      ...jobFragment(),
      variables: [
        { key: "area", type: "TEXT", value: "osaka" },
        { key: "count", type: "NUMBER", value: 3 },
      ],
    });

    expect(converted.variables).toEqual([
      { key: "area", type: "text", value: "osaka" },
      { key: "count", type: "number", value: 3 },
    ]);
  });

  test("keeps the user and trigger that started the job", () => {
    const converted = toJob(
      jobFragment({
        runById: "user-1",
        triggerId: "trigger-1",
        trigger: { id: "trigger-1", description: "Nightly import" },
      }),
    );

    expect(converted.runById).toBe("user-1");
    expect(converted.triggerId).toBe("trigger-1");
    expect(converted.triggerDescription).toBe("Nightly import");
  });

  test("leaves the description unset when the trigger was deleted", () => {
    const converted = toJob(jobFragment({ triggerId: "trigger-1" }));

    expect(converted.runById).toBeUndefined();
    expect(converted.triggerId).toBe("trigger-1");
    expect(converted.triggerDescription).toBeUndefined();
  });
});

describe("toProject", () => {
  const projectFragment = (
    overrides: Partial<ProjectFragment> = {},
  ): ProjectFragment => ({
    id: "project-1",
    name: "Project",
    description: "",
    createdAt: "2026-10-14T00:00:00Z",
    updatedAt: "2026-10-14T00:00:00Z",
    createdById: null,
    updatedById: null,
    workspaceId: "workspace-1",
    sharedToken: null,
    deployment: null,
    isLocked: false,
    alias: "p-project-1",
    visibility: "PUBLIC",
    readme: "",
    license: "",
    topics: [],
    starCount: 0,
    starred: false,
    ...overrides,
  });

  test("keeps the creator and last editor IDs", () => {
    const converted = toProject(
      projectFragment({ createdById: "user-1", updatedById: "user-2" }),
    );

    expect(converted.createdById).toBe("user-1");
    expect(converted.updatedById).toBe("user-2");
  });

  test("drops missing author IDs to undefined", () => {
    const converted = toProject(projectFragment());

    expect(converted.createdById).toBeUndefined();
    expect(converted.updatedById).toBeUndefined();
  });
});

describe("toRunParameterInputs", () => {
  const variable = {
    id: "param-1",
    name: "city",
    type: "text" as const,
    required: true,
    public: false,
    defaultValue: "Tokyo",
  };

  test("sends each variable's value, type and position", () => {
    expect(
      toRunParameterInputs([
        variable,
        {
          ...variable,
          id: "param-2",
          name: "zoom",
          type: "number",
          defaultValue: 0,
        },
      ]),
    ).toEqual([
      {
        id: "param-1",
        name: "city",
        type: "TEXT",
        required: true,
        public: false,
        value: "Tokyo",
        config: undefined,
        index: 0,
      },
      {
        id: "param-2",
        name: "zoom",
        type: "NUMBER",
        required: true,
        public: false,
        value: 0,
        config: undefined,
        index: 1,
      },
    ]);
  });

  test("leaves out variables with no value, since value is non-null", () => {
    expect(
      toRunParameterInputs([
        { ...variable, defaultValue: null },
        { ...variable, defaultValue: undefined },
        { ...variable, defaultValue: "" },
      ]).map((p) => p.value),
    ).toEqual([""]);
  });

  test("sends a type the server has no counterpart for as text", () => {
    expect(
      toRunParameterInputs([{ ...variable, type: "password" }])[0].type,
    ).toBe("TEXT");
  });
});
