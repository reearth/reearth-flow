import type { ApiResponse } from "./api";
import type { Job } from "./job";

/**
 * Coarse attribute type emitted by the engine's `probe-schema` command.
 * Mirrors `AttrType` in engine/runtime/types/src/attr_schema.rs.
 */
export type AttrType =
  | "Bool"
  | "Number"
  | "String"
  | "DateTime"
  | "Array"
  | "Map"
  | "Bytes"
  | "Null"
  | "Unknown";

/** Whether a field is guaranteed present, or only conditionally produced. */
export type AttrPresence = "always" | "maybe";

export type FieldReport = {
  name: string;
  type: AttrType;
  presence: AttrPresence;
};

export type PortReport = {
  /** `true` => the node may emit attributes that cannot be enumerated statically. */
  open: boolean;
  fields: FieldReport[];
};

export type NodeReport = {
  name: string;
  /** Keyed by output port name (e.g. "default"). */
  ports: Record<string, PortReport>;
  note?: string;
};

/**
 * Top-level JSON contract returned by the engine `probe-schema` command,
 * delivered as a GCS artifact (`schema-report.json`) on a completed
 * preview-schema Job. `nodes` is keyed by canvas node id.
 */
export type SchemaReport = {
  version: number;
  sampleSize: number;
  nodes: Record<string, NodeReport>;
};

/**
 * Per-node schema persisted onto a (reader) node's Yjs metadata.
 * Holds the node's output ports as probed by the engine.
 */
export type NodeSchemaMeta = {
  ports: Record<string, PortReport>;
  /** Sample size the engine used for this probe. */
  sampleSize?: number;
  /** ISO timestamp the schema was probed at. */
  sampledAt?: string;
  note?: string;
};

export type PreviewSchema = {
  job?: Job;
} & ApiResponse;
