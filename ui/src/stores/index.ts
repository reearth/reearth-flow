import { atom, useAtom } from "jotai";

import { Project, Workspace } from "@flow/types";

export type DialogType =
  | "welcome-init"
  | "account-settings"
  | "workspaces-settings"
  | "workflow-settings"
  | "general-settings"
  | "keyboard-instructions";

const dialogAtom = atom<DialogType | undefined>(undefined);
export const useDialogAtom = () => useAtom(dialogAtom);

const currentProject = atom<Project | undefined>(undefined);
export const useCurrentProject = () => useAtom(currentProject);

const currentWorkspace = atom<Workspace | undefined>(undefined);
export const useCurrentWorkspace = () => useAtom(currentWorkspace);
