import { describe, expect, test } from "vitest";

import {
  type Diagnostic,
  compareDiagnosticSeverity,
  diagnosticOccurrences,
  diagnosticSeverityRank,
  isAggregatedDiagnostic,
  isFatalDiagnostic,
} from "./diagnostic";

const diagnostic = (overrides: Partial<Diagnostic> = {}): Diagnostic => ({
  code: "geometry_invalid",
  category: "geometry",
  severity: "warn",
  message: "self-intersecting polygon",
  ...overrides,
});

describe("isFatalDiagnostic", () => {
  test("reads effectiveDisposition, not severity", () => {
    expect(
      isFatalDiagnostic(
        diagnostic({ severity: "fatal", effectiveDisposition: "warn_drop" }),
      ),
    ).toBe(false);
    expect(
      isFatalDiagnostic(
        diagnostic({ severity: "warn", effectiveDisposition: "fatal" }),
      ),
    ).toBe(true);
  });

  test("an unresolved disposition is not fatal", () => {
    expect(isFatalDiagnostic(diagnostic())).toBe(false);
  });

  test("reject is not fatal", () => {
    expect(
      isFatalDiagnostic(diagnostic({ effectiveDisposition: "reject" })),
    ).toBe(false);
  });
});

describe("aggregated rows", () => {
  test("a row without a count reports an unknown occurrence count", () => {
    // Reporting 1 here would invent a number: the payload does not say how many
    // times this fired, and "no count" is not the same claim as "once".
    const perFeature = diagnostic({ featureId: "feature-1" });
    expect(isAggregatedDiagnostic(perFeature)).toBe(false);
    expect(diagnosticOccurrences(perFeature)).toBeUndefined();
  });

  test("a terminal fatal row carries a count alongside its feature id", () => {
    // The engine keeps the first fatal per node but counts the rest, so a fatal
    // row names the feature it stopped on *and* says how many failed with it.
    const fatal = diagnostic({
      effectiveDisposition: "fatal",
      featureId: "feature-1",
      aggregatedCount: 5,
      sampleFeatureIds: ["feature-1", "feature-2"],
    });
    expect(isFatalDiagnostic(fatal)).toBe(true);
    expect(diagnosticOccurrences(fatal)).toBe(5);
  });

  test("an aggregated row carries its own count", () => {
    const aggregated = diagnostic({
      aggregatedCount: 1204,
      sampleFeatureIds: ["feature-1", "feature-2"],
    });
    expect(isAggregatedDiagnostic(aggregated)).toBe(true);
    expect(diagnosticOccurrences(aggregated)).toBe(1204);
  });

  test("an aggregated count of zero is still an aggregated row", () => {
    const aggregated = diagnostic({ aggregatedCount: 0 });
    expect(isAggregatedDiagnostic(aggregated)).toBe(true);
    expect(diagnosticOccurrences(aggregated)).toBe(0);
  });
});

describe("severity ordering", () => {
  test("ranks the known severities", () => {
    expect(diagnosticSeverityRank("fatal")).toBeGreaterThan(
      diagnosticSeverityRank("error"),
    );
    expect(diagnosticSeverityRank("warn")).toBeGreaterThan(
      diagnosticSeverityRank("info"),
    );
  });

  test("sorts an unknown severity below every known one instead of dropping it", () => {
    // The engine can emit a severity this build has never heard of; it still
    // has to end up in the list somewhere.
    const sorted = [
      diagnostic({ severity: "quantum" }),
      diagnostic({ severity: "warn" }),
      diagnostic({ severity: "fatal" }),
    ].sort(compareDiagnosticSeverity);

    expect(sorted.map((d) => d.severity)).toEqual(["fatal", "warn", "quantum"]);
  });
});
