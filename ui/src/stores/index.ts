import { atom, useAtom } from "jotai";

export type DialogType =
  | "account-settings"
  | "workspaces-settings"
  | "workflow-settings"
  | "keyboard-settings"
  | "general-settings";

const dialogAtom = atom<DialogType | undefined>(undefined);
export const useDialogAtom = () => useAtom(dialogAtom);
