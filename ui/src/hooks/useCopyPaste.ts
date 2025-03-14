import { useCallback } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { type GeneralState } from "@flow/stores";

export const useCopyPaste = () => {
  const { value: generalState, updateValue: updateGeneralState } =
    useIndexedDB("general");
  const copy = useCallback(
    async (data: GeneralState["clipboard"]) => {
      await updateGeneralState({ clipboard: data });
    },
    [updateGeneralState],
  );

  const cut = useCallback(
    async (data: GeneralState["clipboard"]) => {
      await updateGeneralState({ clipboard: data });
    },
    [updateGeneralState],
  );

  const paste = useCallback(
    async (): Promise<GeneralState["clipboard"] | null> =>
      generalState?.clipboard || null,
    [generalState],
  );

  return {
    copy,
    cut,
    paste,
  };
};
