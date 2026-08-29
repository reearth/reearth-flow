import { atom, useAtom } from "jotai";

import { Workspace, Project, Role } from "@flow/types";

const currentProject = atom<Project | undefined>(undefined);
export const useCurrentProject = () => useAtom(currentProject);

const currentWorkspace = atom<Workspace | undefined>(undefined);
export const useCurrentWorkspace = () => useAtom(currentWorkspace);

const currentUserRole = atom<Role | undefined>(undefined);
export const useCurrentUserRole = () => useAtom(currentUserRole);
