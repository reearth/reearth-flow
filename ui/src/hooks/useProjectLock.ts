import { debounce } from "lodash-es";
import { useCallback, useEffect, useRef, useState } from "react";

import { Project } from "@flow/types";

export default ({
  currentProject,
}: {
  currentProject?: Project | undefined;
}) => {
  const [isLocked, setIsLocked] = useState<boolean>(false);

  // useEffect(() => {
  //   setIsLocked(!!currentProject?.isLocked);
  // }, [currentProject?.isLocked]);

  const useDebouncedCallback = (
    callback: (checked: boolean) => void,
    delay: number,
  ) => {
    const callbackRef = useRef(callback);

    useEffect(() => {
      callbackRef.current = callback;
    }, [callback]);

    return useRef(
      debounce((...args: [boolean]) => callbackRef.current(...args), delay),
    ).current;
  };

  const debouncedHandleLockChange = useDebouncedCallback((checked: boolean) => {
    handleProjectLock(checked);
  }, 2000);

  const handleLockChange = (checked: boolean) => {
    setIsLocked(checked);
    debouncedHandleLockChange(checked);
  };

  const handleProjectLock = useCallback(
    (lock: boolean) => {
      if (!currentProject) return;
      console.log("Lock/Unlock project:", lock);
    },
    [currentProject],
  );

  return {
    isLocked,
    handleProjectLockChange: handleLockChange,
  };
};
