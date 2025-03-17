import { CaretRight } from "@phosphor-icons/react";
import { memo, useEffect, useState } from "react";

import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

const Breadcrumb: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject] = useCurrentProject();
  const [isHovered, setIsHovered] = useState<string[] | undefined>(undefined);

  // This clears selection so that the text doesn't stay selected when hovering over the breadcrumb again
  useEffect(() => {
    if (!isHovered && window.getSelection()) {
      window.getSelection()?.empty();
    }
  }, [isHovered]);

  return (
    <div
      className="flex cursor-default select-none items-center gap-2 px-2 py-1"
      onMouseLeave={() => setIsHovered(undefined)}>
      <p
        className={`max-w-[200px] truncate transition-all delay-0 duration-500 dark:font-thin ${isHovered?.includes("workspace") ? "max-w-[50vw] select-text delay-500" : undefined}`}
        onMouseEnter={() => setIsHovered((h) => [...(h ?? []), "workspace"])}>
        {currentWorkspace?.name}
      </p>
      <p className="dark:font-thin">
        <CaretRight />
      </p>
      <p
        className={`max-w-[200px] truncate transition-all delay-0 duration-500 dark:font-thin ${isHovered?.includes("project") ? "max-w-[50vw] select-text delay-500" : undefined}`}
        onMouseEnter={() => setIsHovered((h) => [...(h ?? []), "project"])}>
        {currentProject?.name}
      </p>
    </div>
  );
};

export default memo(Breadcrumb);
