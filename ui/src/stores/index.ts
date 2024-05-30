import { atom, useAtom } from "jotai";

import { Workspace } from "@flow/lib/gql";
import { Project } from "@flow/types";

export type DialogType =
  | "canvas-search"
  | "account-settings"
  | "workspaces-settings"
  | "workflow-settings"
  | "general-settings"
  | "keyboard-instructions"
  | "add-workspace";

const dialogType = atom<DialogType | undefined>(undefined);
export const useDialogType = () => useAtom(dialogType);

const currentProject = atom<Project | undefined>(undefined);
export const useCurrentProject = () => useAtom(currentProject);

const currentWorkspace = atom<Workspace | undefined>(undefined);
export const useCurrentWorkspace = () => useAtom(currentWorkspace);
