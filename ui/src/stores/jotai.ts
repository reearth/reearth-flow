import { atom, useAtom } from "jotai";

import { Workspace, Project } from "@flow/types";

const currentProject = atom<Project | undefined>(undefined);
export const useCurrentProject = () => useAtom(currentProject);

const currentWorkspace = atom<Workspace | undefined>(undefined);
export const useCurrentWorkspace = () => useAtom(currentWorkspace);

// Transient (in-session) status of reader schema probes, keyed by node id —
// drives the on-node spinner/failure indicator. The probed schema itself lives
// on the node's Yjs metadata; only in-flight/failed probes are tracked here.
export type ReaderSchemaProbeStatus = "running" | "failed";

export type ReaderSchemaProbe = {
  nodeId: string;
  jobId: string;
  status: ReaderSchemaProbeStatus;
};

const readerSchemaProbes = atom<Record<string, ReaderSchemaProbe>>({});
export const useReaderSchemaProbes = () => useAtom(readerSchemaProbes);
