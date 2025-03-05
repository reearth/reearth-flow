import { useCallback } from "react";

import {
  type GeneralState,
  loadStateFromIndexedDB,
  updateClipboardState,
} from "@flow/stores";

export const useCopyPaste = () => {
  const copy = useCallback(async (data: GeneralState["clipboard"]) => {
    await updateClipboardState(data);
  }, []);

  const paste = useCallback(async (): Promise<
    GeneralState["clipboard"] | null
  > => {
    const generalState = await loadStateFromIndexedDB("general");
    return generalState?.clipboard || null;
  }, []);

  return {
    copy,
    paste,
  };
};
