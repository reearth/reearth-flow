import { atom, useAtom } from "jotai";

type DialogType = "account" | "workspaces" | "workflow" | "keyboard" | "settings";

const dialogAtom = atom<DialogType | undefined>(undefined);
export const useDialogAtom = () => useAtom(dialogAtom);
