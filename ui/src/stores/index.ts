import { atom, useAtom } from "jotai";

export type DialogType =
  | "account-settings"
  | "workspaces-settings"
  | "workflow-settings"
  | "general-settings"
  | "keyboard-instructions";

const dialogAtom = atom<DialogType | undefined>(undefined);
export const useDialogAtom = () => useAtom(dialogAtom);
