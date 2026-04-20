import { useCallback, useState } from "react";

import { useDebouncedCallback } from "@flow/hooks";
import { useProject } from "@flow/lib/gql";
import { Project } from "@flow/types";

export default ({
  currentProject,
}: {
  currentProject?: Project | undefined;
}) => {
  const [isLocked, setIsLocked] = useState<boolean>(!!currentProject?.isLocked);
  const { updateProject } = useProject();

  // useEffect(() => {
  //   setIsLocked(!!currentProject?.isLocked);
  // }, [currentProject?.isLocked]);

  const handleProjectLock = useCallback(
    async (lock: boolean) => {
      if (!currentProject) return;
      try {
        await updateProject({
          projectId: currentProject.id,
          isLocked: lock,
        });
      } catch (error) {
        console.error("Failed to update project lock status:", error);
      }
    },
    [currentProject, updateProject],
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
