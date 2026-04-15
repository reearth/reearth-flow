import { useCallback, useState } from "react";

import { useDebouncedCallback } from "@flow/hooks";
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

  const handleProjectLock = useCallback(
    (lock: boolean) => {
      if (!currentProject) return;
      console.log("Lock/Unlock project:", lock);
    },
    [currentProject],
  );

  const debouncedHandleLockChange = useDebouncedCallback((checked: boolean) => {
    handleProjectLock(checked);
  }, 2000);

  const handleLockChange = (checked: boolean) => {
    setIsLocked(checked);
    debouncedHandleLockChange(checked);
  };

  return {
    isLocked,
    handleProjectLockChange: handleLockChange,
  };
};
