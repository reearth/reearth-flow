import { describe, expect, test } from "vitest";

import type {
  DiagnosticFragment,
  JobFragment,
} from "./__gen__/plugins/graphql-request";
import { toDiagnostic, toJob } from "./convert";

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
});
