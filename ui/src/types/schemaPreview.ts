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

export type FieldReport = {
  name: string;
  type: AttrType;
};

export type PortReport = {
  /** `true` => the node may emit attributes that cannot be enumerated statically. */
  open: boolean;
  fields: FieldReport[];
};

export type NodeReport = {
  name: string;
  /** Keyed by output port name (e.g. "features"). */
  ports: Record<string, PortReport>;
  note?: string;
};

export type SchemaReport = {
  version: number;
  sampleSize: number;
  nodes: Record<string, NodeReport>;
};

export type NodeSchemaMeta = {
  ports: Record<string, PortReport>;
  status?: "running" | "failed" | "complete";
  jobId?: string;
  sampleSize?: number;
  sampledAt?: string;
  note?: string;
};

export type PreviewSchema = {
  job?: Job;
} & ApiResponse;
