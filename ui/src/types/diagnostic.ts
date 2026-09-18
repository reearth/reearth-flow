/**
 * A structured diagnostic emitted by the engine: a per-feature error or
 * warning, a `finish()`-time aggregated summary, or a terminal per-node
 * failure.
 *
 * `category`, `severity` and `effectiveDisposition` are plain strings on the
 * wire rather than GraphQL enums, deliberately: a newer engine can emit a
 * value this build has never heard of and it still has to round trip. The cost
 * is that there is no compile-time exhaustiveness check on them, so every
 * consumer below recognises the values we know about and passes anything else
 * through verbatim.
 */
export type Diagnostic = {
  code: string;
  category: string;
  /**
   * Display level only. Never branch on this to decide whether a run passed or
   * failed — use {@link isFatalDiagnostic}, which reads
   * `effectiveDisposition`.
   */
  severity: string;
  /** The authoritative fatality signal. Unresolved (`undefined`) for warn-and-continue. */
  effectiveDisposition?: string;
  nodeId?: string;
  actionType?: string;
  featureId?: string;
  message: string;
  help?: string;
  /**
   * How many times this row's error code fired, on the rows that know: a
   * `finish()`-time roll-up (e.g. "1,204 features dropped, 5 samples") and a
   * node's terminal failure, which counts the features that failed with its
   * code. Absent on a per-feature row reported on its own. Read it structurally
   * — never parse the count out of `message`.
   */
  aggregatedCount?: number;
  sampleFeatureIds?: string[];
};

/**
 * Whether this diagnostic is what failed the run. This is the only correct
 * pass/fail test: a diagnostic can carry `severity: "fatal"` for display while
 * a node-level policy downgraded its disposition, and vice versa.
 */
export const isFatalDiagnostic = (diagnostic: Diagnostic): boolean =>
  diagnostic.effectiveDisposition === "fatal";

/** Whether this row stands for a counted set of occurrences rather than one unquantified event. */
export const isAggregatedDiagnostic = (diagnostic: Diagnostic): boolean =>
  diagnostic.aggregatedCount !== undefined;

/**
 * How many occurrences a row stands for, or `undefined` when the row carries no
 * count.
 *
 * Never substitute 1 for `undefined`. A row without a count is not a row that
 * happened once — it is a row whose payload does not say, and rendering it as 1
 * states something the engine did not. Show it as unknown instead.
 */
export const diagnosticOccurrences = (
  diagnostic: Diagnostic,
): number | undefined => diagnostic.aggregatedCount;

/**
 * Rank used for ordering and for picking the worst diagnostic in a set.
 * Unknown severities sort below every known one rather than being dropped.
 */
const severityRank: Record<string, number> = {
  trace: 0,
  debug: 1,
  info: 2,
  warn: 3,
  error: 4,
  fatal: 5,
};

export const diagnosticSeverityRank = (severity: string): number =>
  severityRank[severity] ?? -1;

/**
 * Whether a severity should read as a failure rather than an advisory. Purely a
 * display decision — pass/fail is {@link isFatalDiagnostic}, which reads
 * `effectiveDisposition`. Shared so a badge and a node marker cannot disagree
 * about what counts as bad.
 */
export const isBlockingSeverity = (severity?: string): boolean =>
  severity === "fatal" || severity === "error";

/** Sorts worst-first, so the row a user needs to read is at the top. */
export const compareDiagnosticSeverity = (
  a: Diagnostic,
  b: Diagnostic,
): number =>
  diagnosticSeverityRank(b.severity) - diagnosticSeverityRank(a.severity);
