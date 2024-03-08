import { useCallback, useState } from "react";

export default <T>(initialValue: T) => {
  const [state, updateState] = useState<T | undefined>(initialValue);

  const handleStateUpdate = useCallback((newState?: T) => updateState(newState), []);

  return [state, handleStateUpdate] as const;
};
