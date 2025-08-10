import { atom, useAtom } from "jotai";

import { Workspace, Project } from "@flow/types";

const currentProject = atom<Project | undefined>(undefined);
export const useCurrentProject = () => useAtom(currentProject);

const currentWorkspace = atom<Workspace | undefined>(undefined);
export const useCurrentWorkspace = () => useAtom(currentWorkspace);

const isSaving = atom<boolean>(false);
export const useIsSaving = () => useAtom(isSaving);
