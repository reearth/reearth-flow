import type { Diagnostic } from "./diagnostic";

/**
 * `pending` is retained for API compatibility but is never emitted by the
 * runtime — don't build UI logic that expects to see it.
 */
export type NodeExecutionStatus =
  | "pending"
  | "starting"
  | "processing"
  | "completed"
  | "failed";

/**
 * One node's execution within a job.
 *
 * The three feature counts are populated per node kind and only once the
 * execution reaches a terminal status. `undefined` means "not applicable, or
 * not finished yet" — it does **not** mean zero, and which of the three is
 * populated is not a reliable way to infer the node's kind.
 */
export type NodeExecution = {
  id: string;
  jobId: string;
  nodeId: string;
  status: NodeExecutionStatus;
  createdAt?: string;
  startedAt?: string;
  completedAt?: string;
  /** Processor nodes, terminal status only. */
  featuresProcessed?: number;
  /** Sink nodes, terminal status only. */
  featuresWritten?: number;
  /** Accumulating processor actions that flush at `finish()`, terminal status only. */
  finishFeatureCount?: number;
  diagnostics?: Diagnostic[];
};
