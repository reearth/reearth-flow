import { atom, useAtom } from "jotai";

import { Workspace, Project } from "@flow/types";

const currentWorkflowId = atom<string | undefined>("main");
export const useCurrentWorkflowId = () => useAtom(currentWorkflowId);

const currentProject = atom<Project | undefined>(undefined);
export const useCurrentProject = () => useAtom(currentProject);

const currentWorkspace = atom<Workspace | undefined>(undefined);
export const useCurrentWorkspace = () => useAtom(currentWorkspace);
