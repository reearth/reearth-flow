import { useState, useCallback } from "react";

export const useCopyPaste = <T>() => {
  const [copiedData, setCopiedData] = useState<T | null>(null);
  const copy = useCallback((data: T) => setCopiedData(data), []);
  const paste = useCallback((): T | null => copiedData, [copiedData]);
  return {
    copiedData,
    copy,
    paste,
  };
};
